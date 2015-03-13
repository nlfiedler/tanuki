#!/usr/bin/env escript
%% -*- erlang -*-
%%! -pa deps/pwd/ebin deps/exif/ebin deps/couchbeam/ebin deps/hackney/ebin deps/idna/ebin deps/jsx/ebin deps/oauth/ebin deps/ssl_verify_hostname/ebin deps/mimetypes/ebin deps/gen_smtp/ebin
%% -------------------------------------------------------------------
%%
%% Copyright (c) 2015 Nathan Fiedler
%%
%% This file is provided to you under the Apache License,
%% Version 2.0 (the "License"); you may not use this file
%% except in compliance with the License. You may obtain
%% a copy of the License at
%%
%% http://www.apache.org/licenses/LICENSE-2.0
%%
%% Unless required by applicable law or agreed to in writing,
%% software distributed under the License is distributed on an
%% "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
%% KIND, either express or implied. See the License for the
%% specific language governing permissions and limitations
%% under the License.
%%
%% -------------------------------------------------------------------
%%
%% Process files in an 'incoming' folder by adding them to CouchDB.
%%
%% This script, which is intended to be run via a cron job, examines the
%% contents of a particular directory, looking for content to add to CouchDB.
%% The parent directories of any files found are used to assign tags to the
%% blobs once the files are stored in CouchDB.
%%
%% See the project home page for details: https://github.com/nlfiedler/tanuki
%%
%% Usage:
%% $ ./incoming.escript -p photos -d tanuki settings.config
%%
%% where settings.config contains the couchdb_url and couchdb_opts terms
%% for connecting to CouchDB, which looks something like this.
%%
%%     {couchdb_url, "http://192.168.1.1:5984"}.
%%     {couchdb_opts, [{basic_auth, {"admin", "secr3t"}}]}.
%%
%% Requires getopt (https://github.com/jcomellas/getopt/) to be installed
%% in /usr/local/lib/erlang/lib (or somewhere in the load path).
%%

-mode(compile).  % gives us a stack trace upon error?

-include_lib("kernel/include/file.hrl").

-define(DBNAME, "tanuki").
-define(EXIF_DATETIME, "EXIF DateTimeOriginal").
-define(DATETIME_FORMAT, "%Y-%m-%d %H:%M").
-define(DATE_REGEX, "(\\d{4})-(\\d{2})-(\\d{2}) (\\d{2}):(\\d{2})").
-define(LOGFILE, "/var/log/tanuki-incoming.log").
-define(V, proplists:get_value).

main(Args) ->
    OptSpecList = [
        {path, $p, "path", string,           "base path of incoming assets"},
        {dest, $d, "dest", string,           "base path of asset storage area"},
        {now,  $N, "now",  {boolean, false}, "consider all paths, even very new ones"}
    ],
    case getopt:parse(OptSpecList, Args) of
        {ok, {Options, NonOptArgs}} ->
            ok = validate_args(Options),
            if length(NonOptArgs) =/= 1 ->
                    io:format("~nMust pass CouchDB configuration file~n~n"),
                    exit(badarg);
                true -> ok
            end,
            Filename = hd(NonOptArgs),
            % Consult the Erlang terms file that contains our configuration,
            case file:consult(Filename) of
                {ok, Terms} ->
                    Incoming = ?V(path, Options),
                    BlobStore = ?V(dest, Options),
                    DoItNow = ?V(now, Options),
                    Logfile = start_logging(),
                    process_incoming(Incoming, BlobStore, DoItNow, Terms),
                    ok = error_logger:logfile(close),
                    email_report(Logfile);
                {error, enoent} ->
                    io:format("File ~p not found", [Filename]),
                    exit(badarg)
                % else, surface the error
            end;
        {error, {Reason, Data}} ->
            io:format("Error: ~s ~p~n~n", [Reason, Data]),
            getopt:usage(OptSpecList, "fixperms")
    end,
    ok.

% Ensure the parsed command line arguments are valid, exiting if not.
validate_args(Options) ->
    ValidatePath = fun(Path) ->
        case filelib:is_dir(Path) of
            true  -> ok;
            false ->
                io:format("Path does not exist: ~s~n", [Path]),
                exit(badarg)
        end
    end,
    ValidatePath(?V(path, Options)),
    % ValidatePath(?V(dest, Options)),
    ok.

% Configure the error_logger to write to a file, returning the path to the file.
start_logging() ->
    Filename = case error_logger:logfile({open, ?LOGFILE}) of
        ok -> ?LOGFILE;
        {error, eacces} ->
            case init:get_argument(home) of
                {ok, Home} ->
                    LogPath = filename:join(lists:flatten(Home), "tanuki-incoming.log"),
                    ok = error_logger:logfile({open, LogPath}),
                    LogPath;
                error ->
                    io:format("Cannot get home directory!"),
                    exit(error)
            end;
        {error, Reason} ->
            io:format("Cannot open log file: ~s~n", [Reason]),
            exit(eaccess)
    end,
    error_logger:tty(false),
    Filename.

% Process the assets in the incoming directory, storing them in the
% blob store area and inserting records in the database.
process_incoming(Incoming, BlobStore, DoItNow, Config) ->
    error_logger:info_msg("Tanuki incoming beginning...~n"),
    ok = application:load(jsx),
    ok = application:load(couchbeam),
    ok = application:start(mimetypes),
    ok = application:start(pwd),
    ok = couchbeam:start(),
    Url = ?V(couchdb_url, Config),
    Opts = case ?V(couchdb_opts, Config) of
        undefined -> [];
        Val -> Val
    end,
    Server = couchbeam:server_connection(Url, Opts),
    {ok, Db} = couchbeam:open_or_create_db(Server, ?DBNAME, []),

    % Get the names of the directories in the Incoming path.
    {ok, Filenames} = file:list_dir(Incoming),
    Filepaths = [filename:join(Incoming, Name) || Name <- Filenames],
    Directories = lists:filter(fun filelib:is_dir/1, Filepaths),
    error_logger:info_msg("Considering the following directories...~n~p~n", [Directories]),
    % Check if the given directory is sufficiently old (at least 1 hour).
    PathOldEnough = fun(Path) ->
        NowSecs = seconds_since_epoch(),
        case file:read_file_info(Path, [{time, posix}]) of
            {ok, #file_info{ctime = CTime}} ->
                NowSecs - CTime > 3600;
            {error, Reason2} ->
                error_logger:error_msg("Failed to read file info of ~s, ~p~n", [Path, Reason2]),
                exit(error)
        end
    end,
    OldEnough = case DoItNow of
        true -> Directories;
        false -> lists:filter(PathOldEnough, Directories)
    end,
    ProcessPath = fun(Path) ->
        process_path(Path, Db, BlobStore)
    end,
    if length(OldEnough) == 0 ->
            error_logger:info_msg("No (old enough) directories found, exiting~n");
        true -> lists:foreach(ProcessPath, OldEnough)
    end,
    ok.

% Import the assets found with the named path.
process_path(Path, Db, BlobStore) ->
    ImportDate = calendar:universal_time(),
    {Tags, Location} = convert_path_to_tags_and_location(Path),
    error_logger:info_msg("Processing tags: ~p~n", [Tags]),
    ok = delete_extraneous_files(Path),
    {ok, Filenames} = file:list_dir(Path),
    ProcFun = fun(Name) ->
        Filepath = filename:join(Path, Name),
        case file:read_file_info(Filepath) of
            {ok, #file_info{type=regular}} ->
                Checksum = compute_checksum(Filepath),
                % Either insert or update a document in the database.
                case find_document(Db, Checksum) of
                    undefined -> create_document(Db, Name, Filepath, Tags, ImportDate,
                                                 Location, Checksum);
                    DocId -> update_document(Db, DocId, Name, Tags, Location)
                end,
                % Move the asset into place, or remove it if duplicate.
                store_asset(Filepath, Checksum, BlobStore);
            _ -> error_logger:warning_msg("Ignoring non-file entry ~s~n", [Name])
        end
    end,
    lists:foreach(ProcFun, Filenames),
    error_logger:info_msg("Done with ~p~n", [Path]),
    case file:del_dir(Path) of
        ok -> ok;
        {error, Reason} ->
            error_logger:error_msg("Unable to remove ~s: ~p~n", [Path, Reason])
    end.

% Convert the path to a set of tags and a location, if any. Location is
% separated from tags by at sign (@). Tags are separated by underscore (_).
% Any underscores in location are replaced with spaces.
convert_path_to_tags_and_location(Path) ->
    AssetFolder = string:to_lower(filename:basename(Path)),
    [Tags|Tail] = re:split(AssetFolder, "@", [{return, list}]),
    Location = case Tail of
        []    -> undefined;
        [L|_] -> re:replace(L, "_", " ")
    end,
    TagList = re:split(Tags, "_", [{return, list}]),
    ValidTags = lists:filter(fun(Tag) -> length(Tag) > 0 end, TagList),
    {ValidTags, Location}.

% Remove any of the superfluous files from the given path.
delete_extraneous_files(Path) ->
    Extras = [".DS_Store", ".localized"],
    {ok, Filenames} = file:list_dir(Path),
    DeleteFun = fun(Name) ->
        case lists:member(Name, Extras) of
            true ->
                case file:delete(filename:join(Path, Name)) of
                    ok -> ok;
                    {error, Reason} ->
                        error_logger:error_msg("File delete failed: ~p~n", [Reason]),
                        exit(ioerr)
                end;
            false -> ok
        end
    end,
    lists:foreach(DeleteFun, Filenames).

% Compute the SHA256 for the named file and return as a hex string.
compute_checksum(Filepath) ->
    case file:read_file(Filepath) of
        {ok, Binary} ->
            Digest = crypto:hash(sha256, Binary),
            lists:flatten([io_lib:format("~2.16.0B", [X]) || <<X:8>> <= Digest]);
        {error, Reason} ->
            error_logger:error_msg("File read failed: ~p~n", [Reason]),
            exit(ioerr)
    end.

% Determine if the given checksum already exists in the database.
% Returns the document id (binary), or undefined if not found.
find_document(Db, Checksum) ->
    % check if view is defined and return immediately if not
    case couchbeam:doc_exists(Db, "_design/assets") of
        true ->
            Options = [{key, list_to_binary(Checksum)}],
            case couchbeam_view:fetch(Db, {"assets", "by_checksum"}, Options) of
                {ok, Rows} ->
                    if length(Rows) == 1 -> couchbeam_doc:get_value(<<"id">>, hd(Rows));
                        true -> undefined
                    end;
                {error, Reason} ->
                    error_logger:error_msg("couchdb fetch failed: ~p~n", [Reason]),
                    undefined
            end;
        false -> undefined
    end.

% Create a new document in the CouchDB database.
create_document(Db, Filename, Fullpath, Tags, ImportDate, Location, Checksum) ->
    error_logger:info_msg("Creating document for ~s...~n", [Filename]),
    {{Y, Mo, D}, {H, Mi, _S}} = ImportDate,
    LocData = case Location of
        undefined -> null;
        Value -> list_to_binary(Value)
    end,
    Doc = {[
        {<<"exif_date">>, get_original_exif_date(Fullpath)},
        {<<"file_date">>, file_date(Fullpath)},
        {<<"file_name">>, list_to_binary(Filename)},
        {<<"file_owner">>, list_to_binary(file_owner(Fullpath))},
        {<<"file_size">>, file_size(Fullpath)},
        {<<"import_date">>, [Y, Mo, D, H, Mi]},
        {<<"location">>, LocData},
        {<<"mimetype">>, hd(mimetypes:filename(Filename))},
        {<<"sha256">>, list_to_binary(Checksum)},
        {<<"tags">>, [list_to_binary(Tag) || Tag <- Tags]}
    ]},
    case couchbeam:save_doc(Db, Doc) of
        {ok, NewDoc} ->
            {Id, Rev} = couchbeam_doc:get_idrev(NewDoc),
            error_logger:info_msg("~s => id=~s, rev=~s~n",
                                  [Filename, binary_to_list(Id), binary_to_list(Rev)]);
        {error, Reason} ->
            error_logger:error_msg("document save failed: ~p~n", [Reason]),
            exit(error)
    end,
    ok.

% Merge the given tags with existing document's tags.
% If missing location field, set to location argument.
update_document(Db, DocId, Filename, Tags, Location) ->
    error_logger:info_msg("Updating document for ~s...~n", [Filename]),
    {ok, Doc} = couchbeam:open_doc(Db, DocId),
    Doc1 = case get_field_value(<<"location">>, Doc) of
        none -> couchbeam_doc:set_value(<<"location">>, Location, Doc);
        _Val -> Doc
    end,
    BinTags = [list_to_binary(Tag) || Tag <- Tags],
    error_logger:info_msg("new tags: ~p~n", [BinTags]),
    case get_field_value(<<"tags">>, Doc1) of
        none ->
            Doc2 = couchbeam_doc:set_value(<<"tags">>, BinTags, Doc1);
        OldTags ->
            error_logger:info_msg("old tags: ~p~n", [OldTags]),
            MergedTags = lists:umerge(lists:sort(OldTags), lists:sort(BinTags)),
            error_logger:info_msg("merged tags: ~p~n", [MergedTags]),
            Doc2 = couchbeam_doc:set_value(<<"tags">>, MergedTags, Doc1)
    end,
    {ok, NewDoc} = couchbeam:save_doc(Db, Doc2),
    {Id, Rev} = couchbeam_doc:get_idrev(NewDoc),
    error_logger:info_msg("~s => id=~s, rev=~s~n",
                          [Filename, binary_to_list(Id), binary_to_list(Rev)]),
    ok.

% Retrieve the named field from the document, returning none if not present.
get_field_value(Field, Document) ->
    % This is duplicated in tanuki_backend.erl...
    case couchbeam_doc:get_value(Field, Document) of
        undefined ->
            none;
        null ->
            none;
        Value ->
            Value
    end.

% Return the username of the file owner, of undefined if not available.
file_owner(Path) ->
    case file:read_file_info(Path) of
        {ok, #file_info{uid = UserID}} ->
            Details = pwd:getpwuid(UserID),
            ?V(pw_name, Details);
        {error, Reason} ->
            error_logger:error_msg("Failed to read file info of ~s, ~p~n", [Path, Reason]),
            exit(error)
    end.

% Return the size in bytes of the named file.
file_size(Path) ->
    case file:read_file_info(Path) of
        {ok, #file_info{size = Size}} ->
            Size;
        {error, Reason} ->
            error_logger:error_msg("Failed to read file info of ~s, ~p~n", [Path, Reason]),
            exit(error)
    end.

% Return the mtime of the file, which is typically when it was created.
file_date(Path) ->
    case file:read_file_info(Path) of
        {ok, #file_info{mtime = Mtime}} ->
            {{Y, Mo, D}, {H, Mi, _S}} = Mtime,
            [Y, Mo, D, H, Mi];
        {error, Reason} ->
            error_logger:error_msg("Failed to read file info of ~s, ~p~n", [Path, Reason]),
            exit(error)
    end.

% Attempt to read the original datetime from the EXIF tags, returning
% null if not available, or the date time as a list of integers.
get_original_exif_date(Path) ->
    case exif:read(Path) of
        {error, Reason} ->
            error_logger:error_msg("Unable to read EXIF data from ~s, ~p~n", [Path, Reason]),
            null;
        ExifDate ->
            case dict:find(date_time_original, ExifDate) of
                {ok, Original} ->
                    Parsed = date_parse(Original),
                    {{Y, Mo, D}, {H, Mi, _S}} = Parsed,
                    [Y, Mo, D, H, Mi];
                error ->
                    null
            end
    end.

% Move the named asset to its sharded location.
store_asset(Filepath, Checksum, BlobStore) ->
    % If an existing asset with the same checksum already exists, the new
    % asset will be removed to prevent further processing in the future.
    Part1 = string:sub_string(Checksum, 1, 2),
    Part2 = string:sub_string(Checksum, 3, 4),
    Part3 = string:sub_string(Checksum, 5),
    BlobPath = filename:join([BlobStore, Part1, Part2]),
    DestPath = filename:join(BlobPath, Part3),
    case filelib:is_dir(BlobPath) of
        true  -> ok;
        false -> filelib:ensure_dir(DestPath)
    end,
    error_logger:info_msg("Moving ~s to ~s~n", [Filepath, DestPath]),
    case filelib:is_regular(DestPath) of
        true  -> ok = file:delete(Filepath);
        false -> ok = file:rename(Filepath, DestPath)
    end.

% Send the text in the named file via email to the root user.
% Failing that, dump the content to stdout.
email_report(Filename) ->
    % Use io:format() to report errors as the log file is already closed.
    case file:read_file(Filename) of
        {ok, Binary} ->
            Subject = <<"Subject: incoming processor report\r\n">>,
            From = <<"From: tanuki\r\n">>,
            To = <<"To: root\r\n\r\n">>,
            Body = <<Subject/binary, From/binary, To/binary, Binary/binary>>,
            Email = {<<"tanuki">>, [<<"root">>], Body},
            Options = [{relay, "localhost"}],
            case gen_smtp_client:send_blocking(Email, Options) of
                {error, Type, Message} ->
                    io:format("Email send failed: ~p, ~p~n", [Type, Message]);
                {error, Reason} ->
                    io:format("Email send failed: ~p~n", [Reason]);
                _Receipt ->
                    ok
            end;
        {error, Reason} ->
            io:format("File read failed: ~p~n", [Reason]),
            exit(ioerr)
    end,
    ok.

% Parse a simple date/time string into a date/time tuple. For instance,
% <<"2014:10:11 13:28:00">> would become {{2014,10,11},{13,28,0}}. A
% simplified version of https://github.com/seansawyer/erlang_iso8601 for
% this different date/time format.
date_parse(Bin) when is_binary(Bin) ->
    date_parse(binary_to_list(Bin));
date_parse(Str) ->
    parse_year(Str, []).

parse_year([Y1,Y2,Y3,Y4|Rest], Acc) ->
    acc([Y1,Y2,Y3,Y4], Rest, year, Acc, fun parse_month/2);
parse_year(_, _) ->
    error(badarg).

parse_month([], Acc) ->
    datetime(Acc);
parse_month([$:,M1,M2|Rest], Acc) ->
    acc([M1,M2], Rest, month, Acc, fun parse_day/2);
parse_month(_, _) ->
    error(badarg).

parse_day([], Acc) ->
    datetime(Acc);
parse_day([$:,D1,D2|Rest], Acc) ->
    acc([D1,D2], Rest, day, Acc, fun parse_hour/2);
parse_day(_, _) ->
    error(badarg).

parse_hour([], Acc) ->
    datetime(Acc);
parse_hour([$\ ,H1,H2|Rest], Acc) ->
    acc([H1,H2], Rest, hour, Acc, fun parse_minute/2);
parse_hour(_, _) ->
    error(badarg).

parse_minute([], Acc) ->
    datetime(Acc);
parse_minute([$:,M1,M2|Rest], Acc) ->
    acc([M1,M2], Rest, minute, Acc, fun parse_second/2);
parse_minute(_, _) ->
    error(badarg).

parse_second([], Acc) ->
    datetime(Acc);
parse_second([$:,S1,S2|Rest], Acc) ->
    acc([S1,S2], Rest, second, Acc, fun datetime/2);
parse_second(_, _) ->
    error(badarg).

acc(IntStr, Rest, Key, Acc, NextF) ->
    Acc1 = [{Key, list_to_integer(IntStr)}|Acc],
    NextF(Rest, Acc1).

datetime(Plist) ->
    Date = {?V(year, Plist, 0), ?V(month, Plist, 0), ?V(day, Plist, 0)},
    Time = {?V(hour, Plist, 0), ?V(minute, Plist, 0), ?V(second, Plist, 0)},
    {Date, Time}.

datetime(_, Plist) ->
    datetime(Plist).

seconds_since_epoch() ->
    {Mega, Sec, _Micro} = os:timestamp(),
    Mega * 1000000 + Sec.
