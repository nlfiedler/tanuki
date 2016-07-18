%% -*- coding: utf-8 -*-
%% -------------------------------------------------------------------
%%
%% Copyright (c) 2015-2016 Nathan Fiedler
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
    setup_timer(),
    State = #state{
        server=Server,
        database=Db,
        incoming_dir=IncomingPath,
        blob_store=BlobStore
    },
    {ok, State}.

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
        {ok, #file_info{ctime = CTime}} = file:read_file_info(Path, [{time, posix}]),
        NowSecs - CTime > 3600
    end,
    process_incoming(Incoming, Store, Db, FilterFun),
    {noreply, State};
handle_cast(_Msg, State) ->
    {noreply, State}.

handle_info(Msg, State) ->
    lager:notice("unexpected message: ~p", [Msg]),
    {noreply, State}.

terminate(_Reason, _State) ->
    ok.

code_change(_OldVsn, State, _Extra) ->
    {ok, State}.

%%
%% internal functions
%%

% Start a timer to cast a 'process' message to ourselves every hour.
setup_timer() ->
    M = gen_server,
    F = cast,
    A = [tanuki_incoming, process],
    {ok, _TRef} = timer:apply_interval(1000*60*60, M, F, A),
    ok.

% Process the assets in the incoming directory, storing them in the
% blob store area and inserting records in the database.
process_incoming(Incoming, BlobStore, Db, FilterFun) ->
    lager:info("Incoming asset processing commencing..."),
    % Get the names of the directories in the Incoming path.
    {ok, Filenames} = file:list_dir(Incoming),
    Filepaths = [filename:join(Incoming, Name) || Name <- Filenames],
    Directories = lists:filter(fun filelib:is_dir/1, Filepaths),
    lager:info("Considering the following directories: ~p", [Directories]),
    % Check if the given directory is sufficiently old.
    OldEnough = lists:filter(FilterFun, Directories),
    ProcessPath = fun(Path) ->
        process_path(Path, BlobStore, Db)
    end,
    if length(OldEnough) == 0 ->
            lager:info("No (old enough) directories found at this time");
        true -> lists:foreach(ProcessPath, OldEnough)
    end,
    ok.

% Import the assets found with the named path.
process_path(Path, BlobStore, Db) ->
    ImportDate = calendar:universal_time(),
    {Topic, Tags, Location} = convert_path_to_details(Path),
    lager:info("Processing tags: ~p", [Tags]),
    ok = delete_extraneous_files(Path),
    {ok, Filenames} = file:list_dir(Path),
    ProcFun = fun(Name) ->
        Filepath = filename:join(Path, Name),
        case file:read_file_info(Filepath) of
            {ok, #file_info{type=regular}} ->
                Binary = correct_orientation(Filepath),
                Checksum = compute_checksum(Binary),
                % Either insert or update a document in the database.
                case find_document(Db, Checksum) of
                    undefined -> create_document(Db, Name, Filepath, Tags, ImportDate,
                                                 Topic, Location, Checksum, Binary);
                    DocId -> update_document(Db, DocId, Name, Tags, Topic, Location)
                end,
                % Move the asset into place, or remove it if duplicate.
                store_asset(Filepath, Checksum, Binary, BlobStore);
            _ -> lager:warning("Ignoring non-file entry ~s", [Name])
        end
    end,
    lists:foreach(ProcFun, Filenames),
    lager:info("Done with ~p", [Path]),
    case file:del_dir(Path) of
        ok -> ok;
        {error, Reason} ->
            lager:error("failed to remove ~s: ~p", [Path, Reason])
    end.

% Convert the path to a topic, set of tags, and a location. Topic, if any,
% is separated by a circumflex (^) and starts at the beginning of the path.
% Location, if any, starts with an at sign (@) and goes to the end of the
% path. Tags are required and are separated by underscore (_). Any
% underscores in topic and location are replaced with spaces.
convert_path_to_details(Path) ->
    AssetFolder = string:to_lower(filename:basename(Path)),
    [MaybeTopic|Rest] = re:split(AssetFolder, "\\^", [{return, list}]),
    case Rest of
        []     ->
            Topic = undefined,
            NotTopic = MaybeTopic;
        [R|_] ->
            Topic = re:replace(MaybeTopic, "_", " ", [global]),
            NotTopic = R
    end,
    [Tags|Tail] = re:split(NotTopic, "@", [{return, list}]),
    Location = case Tail of
        []    -> undefined;
        [L|_] -> re:replace(L, "_", " ", [global])
    end,
    TagList = re:split(Tags, "_", [{return, list}]),
    ValidTags = lists:filter(fun(Tag) -> length(Tag) > 0 end, TagList),
    {Topic, ValidTags, Location}.

% Remove any of the superfluous files from the given path.
delete_extraneous_files(Path) ->
    % Annoying and basically useless files specific to Mac OS X.
    Extras = [".DS_Store", ".localized"],
    {ok, Filenames} = file:list_dir(Path),
    DeleteFun = fun(Name) ->
        case lists:member(Name, Extras) of
            true ->
                FilePath = filename:join(Path, Name),
                case file:delete(FilePath) of
                    ok -> ok;
                    {error, Reason} ->
                        lager:error("failed to delete file ~w, ~w", [FilePath, Reason])
                end;
            false -> ok
        end
    end,
    lists:foreach(DeleteFun, Filenames).

% Load the named asset file, correct its orientation if applicable, and
% return the binary data. Does not modify the asset if the auto-orientation
% is not successful.
correct_orientation(Filepath) ->
    % Fail fast if unable to read file.
    {ok, Binary0} = file:read_file(Filepath),
    case emagick_rs:requires_orientation(Binary0) of
        false ->
            % Return the original binary, rather than the one that has been
            % handled by ImageMagick, which invariably modifies the data,
            % and thus changes the checksum.
            Binary0;
        true ->
            % Attempt to auto-orient the image, but tolerate assets that
            % are not images, and hence cannot be rotated.
            case emagick_rs:auto_orient(Binary0) of
                {ok, Binary1} -> Binary1;
                {error, Reason} ->
                    lager:warning("unable to auto-orient asset: ~p", [Reason]),
                    Binary0
            end
    end.

% Compute the SHA256 for the given binary and return as a hex string.
compute_checksum(Binary) ->
    Digest = crypto:hash(sha256, Binary),
    lists:flatten([io_lib:format("~2.16.0b", [X]) || <<X:8>> <= Digest]).

% Determine if the given checksum already exists in the database.
% Returns the document id (binary), or undefined if not found.
find_document(Db, Checksum) ->
    % check if view is defined and return immediately if not
    case couchbeam:doc_exists(Db, "_design/assets") of
        true ->
            Options = [{key, list_to_binary(Checksum)}],
            % Fail fast if unable to check for duplicates, rather than
            % blindly creating more duplicate records.
            {ok, Rows} = couchbeam_view:fetch(Db, {"assets", "by_checksum"}, Options),
            % Likewise fail fast if there are multiple records with matching
            % checksums, as that indicates an existing problem that needs
            % immediate attention.
            case length(Rows) of
                1 -> couchbeam_doc:get_value(<<"id">>, hd(Rows));
                0 -> undefined
            end;
        false -> undefined
    end.

% Create a new document in the CouchDB database.
create_document(Db, Filename, Fullpath, Tags, ImportDate, Topic, Location, Checksum, Binary) ->
    lager:info("Creating document for ~s", [Filename]),
    {{Y, Mo, D}, {H, Mi, _S}} = ImportDate,
    LocData = case Location of
        undefined -> null;
        Value1 -> list_to_binary(Value1)
    end,
    TopData = case Topic of
        undefined -> null;
        Value2 -> list_to_binary(Value2)
    end,
    Doc = {[
        {<<"exif_date">>, get_original_exif_date(Binary)},
        {<<"file_date">>, file_date(Fullpath)},
        {<<"file_name">>, list_to_binary(Filename)},
        {<<"file_owner">>, list_to_binary(file_owner(Fullpath))},
        {<<"file_size">>, byte_size(Binary)},
        {<<"import_date">>, [Y, Mo, D, H, Mi]},
        {<<"location">>, LocData},
        {<<"mimetype">>, hd(mimetypes:filename(string:to_lower(Filename)))},
        {<<"sha256">>, list_to_binary(Checksum)},
        {<<"tags">>, [list_to_binary(Tag) || Tag <- lists:sort(Tags)]},
        {<<"topic">>, TopData}
    ]},
    % Fail fast if insertion failed, so we do not then move the asset out
    % of the incoming directory and fail to process it properly.
    {ok, NewDoc} = couchbeam:save_doc(Db, Doc),
    {Id, Rev} = couchbeam_doc:get_idrev(NewDoc),
    lager:info("~s => id=~s, rev=~s", [Filename, binary_to_list(Id), binary_to_list(Rev)]),
    ok.

% Merge the given tags with existing document's tags.
% If missing topic field, set to Topic argument.
% If missing location field, set to Location argument.
update_document(Db, DocId, Filename, Tags, Topic, Location) ->
    lager:info("Updating document for ~s", [Filename]),
    {ok, Doc} = couchbeam:open_doc(Db, DocId),
    Doc1 = maybe_set_location(Doc, Location),
    Doc2 = maybe_set_topic(Doc1, Topic),
    BinTags = [list_to_binary(Tag) || Tag <- Tags],
    lager:info("new tags: ~p", [BinTags]),
    case tanuki_backend:get_field_value(<<"tags">>, Doc2) of
        none ->
            Doc3 = couchbeam_doc:set_value(<<"tags">>, BinTags, Doc2);
        OldTags ->
            lager:info("old tags: ~p", [OldTags]),
            MergedTags = lists:umerge(lists:sort(OldTags), lists:sort(BinTags)),
            lager:info("merged tags: ~p", [MergedTags]),
            Doc3 = couchbeam_doc:set_value(<<"tags">>, MergedTags, Doc2)
    end,
    {ok, NewDoc} = couchbeam:save_doc(Db, Doc3),
    {Id, Rev} = couchbeam_doc:get_idrev(NewDoc),
    lager:info("~s => id=~s, rev=~s", [Filename, binary_to_list(Id), binary_to_list(Rev)]),
    ok.

% Set the location field in the document, if not already set.
% Returns the updated (or not) document.
maybe_set_location(Doc, Location) ->
    Value = case Location of
        undefined -> null;
        V -> list_to_binary(V)
    end,
    NewDoc = case tanuki_backend:get_field_value(<<"location">>, Doc) of
        none -> couchbeam_doc:set_value(<<"location">>, Value, Doc);
        _V -> Doc
    end,
    NewDoc.

% Set the topic field in the document, if not already set.
% Returns the updated (or not) document.
maybe_set_topic(Doc, Topic) ->
    Value = case Topic of
        undefined -> null;
        V -> list_to_binary(V)
    end,
    NewDoc = case tanuki_backend:get_field_value(<<"topic">>, Doc) of
        none -> couchbeam_doc:set_value(<<"topic">>, Value, Doc);
        _V -> Doc
    end,
    NewDoc.

% Return the username of the file owner, of undefined if not available.
file_owner(Path) ->
    {ok, #file_info{uid = UserID}} = file:read_file_info(Path),
    {ok, Details} = epwd_rs:getpwuid(UserID),
    proplists:get_value(pw_name, Details).

% Return the mtime of the file, which is typically when it was created.
file_date(Path) ->
    {ok, #file_info{mtime = Mtime}} = file:read_file_info(Path),
    {{Y, Mo, D}, {H, Mi, _S}} = Mtime,
    [Y, Mo, D, H, Mi].

% Attempt to read the original datetime from the EXIF tags, returning
% null if not available, or the date time as a list of integers.
get_original_exif_date(ImageData) ->
    case emagick_rs:image_get_property(ImageData, "exif:DateTimeOriginal") of
        {error, Reason} ->
            lager:warning("unable to read EXIF data: ~p", [Reason]),
            null;
        {ok, OriginalDate} ->
            Parsed = date_parse(OriginalDate),
            {{Y, Mo, D}, {H, Mi, _S}} = Parsed,
            [Y, Mo, D, H, Mi]
    end.

% Move the named asset to its sharded location.
store_asset(Filepath, Checksum, Binary, BlobStore) ->
    % If an existing asset with the same checksum already exists, the new
    % asset will be removed to prevent further processing in the future.
    Part1 = string:sub_string(Checksum, 1, 2),
    Part2 = string:sub_string(Checksum, 3, 4),
    Part3 = string:sub_string(Checksum, 5),
    BlobPath = filename:join([BlobStore, Part1, Part2]),
    DestPath = filename:join(BlobPath, Part3),
    case filelib:is_dir(BlobPath) of
        true  -> ok;
        false -> ok = filelib:ensure_dir(DestPath)
    end,
    lager:info("Moving ~s to ~s", [Filepath, DestPath]),
    case filelib:is_regular(DestPath) of
        true -> ok = file:delete(Filepath);
        false ->
            ok = file:write_file(DestPath, Binary),
            ok = file:delete(Filepath)
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
