%% -*- coding: utf-8 -*-
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
-module(tanuki_incoming).
-behavior(gen_server).
-export([start_link/0]).
-export([init/1, handle_call/3, handle_cast/2, handle_info/2, terminate/2, code_change/3]).

-include_lib("kernel/include/file.hrl").

-record(state, {server, database, incoming_dir, blob_store}).

-define(DELAY, 1000*60*60).

%%
%% Client API
%%
start_link() ->
    gen_server:start_link({local, ?MODULE}, ?MODULE, [], []).

%%
%% gen_server callbacks
%%
init([]) ->
    {ok, IncomingPath} = application:get_env(tanuki_incoming, incoming_dir),
    {ok, BlobStore} = application:get_env(tanuki_incoming, assets_dir),
    {ok, Url} = application:get_env(tanuki_incoming, couchdb_url),
    {ok, Opts} = application:get_env(tanuki_incoming, couchdb_opts),
    {ok, DbName} = application:get_env(tanuki_incoming, database),
    Server = couchbeam:server_connection(Url, Opts),
    {ok, Db} = couchbeam:open_or_create_db(Server, DbName, []),
    State = #state{
        server=Server,
        database=Db,
        incoming_dir=IncomingPath,
        blob_store=BlobStore
    },
    fire_later(),
    {ok, State}.

handle_call(terminate, _From, State) ->
    {stop, normal, ok, State};
handle_call(process_now, _From, State) ->
    % Primarily for testing, so the test code can block until a (useless)
    % response is received.
    #state{database=Db, incoming_dir=Incoming, blob_store=Store} = State,
    % All directories are old enough as we want to process them now.
    FilterFun = fun(_Path) -> true end,
    process_incoming(Incoming, Store, Db, FilterFun),
    {reply, ok, State}.

handle_cast(process, State) ->
    #state{database=Db, incoming_dir=Incoming, blob_store=Store} = State,
    % Filter function to ensure the given directory is at least an hour old.
    FilterFun = fun(Path) ->
        NowSecs = tanuki_backend:seconds_since_epoch(),
        case file:read_file_info(Path, [{time, posix}]) of
            {ok, #file_info{ctime = CTime}} ->
                NowSecs - CTime > 3600;
            {error, Reason2} ->
                error_logger:error_msg("Failed to read file info of ~s, ~p~n", [Path, Reason2]),
                false
        end
    end,
    process_incoming(Incoming, Store, Db, FilterFun),
    fire_later(),
    {noreply, State};
handle_cast(_Msg, State) ->
    {noreply, State}.

handle_info(Msg, State) ->
    error_logger:info_msg("unexpected message: ~p~n", [Msg]),
    {noreply, State}.

terminate(_Reason, _State) ->
    ok.

code_change(_OldVsn, State, _Extra) ->
    {ok, State}.

%%
%% internal functions
%%

% Start a timer to cast a 'process' message to us later.
fire_later() ->
    M = gen_server,
    F = cast,
    A = [tanuki_incoming, process],
    {ok, _TRef} = timer:apply_after(?DELAY, M, F, A),
    ok.

% Process the assets in the incoming directory, storing them in the
% blob store area and inserting records in the database.
process_incoming(Incoming, BlobStore, Db, FilterFun) ->
    error_logger:info_msg("Incoming asset processing commencing...~n"),
    % Get the names of the directories in the Incoming path.
    {ok, Filenames} = file:list_dir(Incoming),
    Filepaths = [filename:join(Incoming, Name) || Name <- Filenames],
    Directories = lists:filter(fun filelib:is_dir/1, Filepaths),
    error_logger:info_msg("Considering the following directories...~n~p~n", [Directories]),
    % Check if the given directory is sufficiently old.
    OldEnough = lists:filter(FilterFun, Directories),
    ProcessPath = fun(Path) ->
        process_path(Path, BlobStore, Db)
    end,
    if length(OldEnough) == 0 ->
            error_logger:info_msg("No (old enough) directories found at this time~n");
        true -> lists:foreach(ProcessPath, OldEnough)
    end,
    ok.

% Import the assets found with the named path.
process_path(Path, BlobStore, Db) ->
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
    % Annoying and basically useless files specific to Mac OS X.
    Extras = [".DS_Store", ".localized"],
    {ok, Filenames} = file:list_dir(Path),
    DeleteFun = fun(Name) ->
        case lists:member(Name, Extras) of
            true ->
                case file:delete(filename:join(Path, Name)) of
                    ok -> ok;
                    {error, Reason} ->
                        error_logger:error_msg("File delete failed: ~p~n", [Reason])
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
            lists:flatten([io_lib:format("~2.16.0b", [X]) || <<X:8>> <= Digest]);
        {error, Reason} ->
            error_logger:error_msg("File read failed: ~p~n", [Reason]),
            undefined
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
        {<<"mimetype">>, hd(mimetypes:filename(string:to_lower(Filename)))},
        {<<"sha256">>, list_to_binary(Checksum)},
        {<<"tags">>, [list_to_binary(Tag) || Tag <- Tags]}
    ]},
    case couchbeam:save_doc(Db, Doc) of
        {ok, NewDoc} ->
            {Id, Rev} = couchbeam_doc:get_idrev(NewDoc),
            error_logger:info_msg("~s => id=~s, rev=~s~n",
                                  [Filename, binary_to_list(Id), binary_to_list(Rev)]);
        {error, Reason} ->
            error_logger:error_msg("document save failed: ~p~n", [Reason])
    end,
    ok.

% Merge the given tags with existing document's tags.
% If missing location field, set to location argument.
update_document(Db, DocId, Filename, Tags, Location) ->
    error_logger:info_msg("Updating document for ~s...~n", [Filename]),
    {ok, Doc} = couchbeam:open_doc(Db, DocId),
    Doc1 = case tanuki_backend:get_field_value(<<"location">>, Doc) of
        none -> couchbeam_doc:set_value(<<"location">>, Location, Doc);
        _Val -> Doc
    end,
    BinTags = [list_to_binary(Tag) || Tag <- Tags],
    error_logger:info_msg("new tags: ~p~n", [BinTags]),
    case tanuki_backend:get_field_value(<<"tags">>, Doc1) of
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

% Return the username of the file owner, of undefined if not available.
file_owner(Path) ->
    case file:read_file_info(Path) of
        {ok, #file_info{uid = UserID}} ->
            Details = pwd:getpwuid(UserID),
            proplists:get_value(pw_name, Details);
        {error, Reason} ->
            error_logger:error_msg("Failed to read file info of ~s, ~p~n", [Path, Reason]),
            undefined
    end.

% Return the size in bytes of the named file.
file_size(Path) ->
    case file:read_file_info(Path) of
        {ok, #file_info{size = Size}} ->
            Size;
        {error, Reason} ->
            error_logger:error_msg("Failed to read file info of ~s, ~p~n", [Path, Reason]),
            undefined
    end.

% Return the mtime of the file, which is typically when it was created.
file_date(Path) ->
    case file:read_file_info(Path) of
        {ok, #file_info{mtime = Mtime}} ->
            {{Y, Mo, D}, {H, Mi, _S}} = Mtime,
            [Y, Mo, D, H, Mi];
        {error, Reason} ->
            error_logger:error_msg("Failed to read file info of ~s, ~p~n", [Path, Reason]),
            undefined
    end.

% Attempt to read the original datetime from the EXIF tags, returning
% null if not available, or the date time as a list of integers.
get_original_exif_date(Path) ->
    case exif:read(Path) of
        {error, Reason} ->
            error_logger:error_msg("Unable to read EXIF data from ~s, ~p~n", [Path, Reason]),
            null;
        ExifData ->
            case dict:find(date_time_original, ExifData) of
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
    Date = {
        proplists:get_value(year, Plist, 0),
        proplists:get_value(month, Plist, 0),
        proplists:get_value(day, Plist, 0)
    },
    Time = {
        proplists:get_value(hour, Plist, 0),
        proplists:get_value(minute, Plist, 0),
        proplists:get_value(second, Plist, 0)
    },
    {Date, Time}.

datetime(_, Plist) ->
    datetime(Plist).
