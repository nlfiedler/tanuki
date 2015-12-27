%% -*- coding: utf-8 -*-
%% -------------------------------------------------------------------
%%
%% Copyright (c) 2014 Nathan Fiedler
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
-module(tanuki_backend).
-export([by_checksum/1, by_date/1, by_date/2, by_tag/1, by_tags/1]).
-export([all_tags/0, fetch_document/1, path_to_mimes/1, generate_etag/3]).
-export([get_best_date/1, date_list_to_string/1, date_list_to_string/2]).
-export([retrieve_thumbnail/2, get_field_value/2, seconds_since_epoch/0]).
-include_lib("tanuki_backend/include/records.hrl").

%%
%% Client API
%%

%
% @doc Retrieves the document with the given identifier as an ejson object
%      (where ejson is defined in couchbeam as essentially a proplist of
%      binary strings).
%
-spec fetch_document(DocId) -> {ok, Doc} | {error, Reason}
    when DocId  :: string(),
         Doc    :: term(),
         Reason :: term().
fetch_document(DocId) ->
    gen_server:call(tanuki_backend_db, {fetch_document, DocId}).

%
% @doc Retrieves all known tags as couchbeam view results.
%
-spec all_tags() -> Rows
    when Rows :: [term()].
all_tags() ->
    gen_server:call(tanuki_backend_db, all_tags).

%
% @doc Retrieves all documents with a given checksum, as couchbeam view results.
%      Result includes the id and mimetype fields.
%
-spec by_checksum(Checksum) -> Rows
    when Checksum :: string(),
         Rows     :: [term()].
by_checksum(Checksum) when is_list(Checksum) ->
    gen_server:call(tanuki_backend_db, {by_checksum, Checksum}).

%
% @doc Retrieves all documents with a given tag, as couchbeam view results.
%
-spec by_tag(Tag) -> Rows
    when Tag  :: string(),
         Rows :: [term()].
by_tag(Tag) when is_list(Tag) ->
    gen_server:call(tanuki_backend_db, {by_tag, Tag}).

%
% @doc Retrieves all documents with the given tags, as couchbeam view results.
%      Only those documents containing all of the given tags will be returned.
%      Ordering is non-deterministic.  TODO: that needs to be fixed
%
-spec by_tags(Tags) -> Rows
    when Tags :: [string()],
         Rows :: [term()].
by_tags(Tags) when is_list(Tags) ->
    % TODO: check for cached results in mnesia
    % TODO: cache the raw results (AllRows) in mnesia
    % TODO: do our own start_id/start_key scanning for pagination support
    AllRows = gen_server:call(tanuki_backend_db, {by_tags, Tags}),
    % Reduce the results to those that have all of the given tags.
    TagCounts = lists:foldl(fun(Row, AccIn) ->
            DocId = couchbeam_doc:get_value(<<"id">>, Row),
            Count = maps:get(DocId, AccIn, 0),
            maps:put(DocId, Count + 1, AccIn)
        end, #{}, AllRows),
    MatchingRows = lists:filter(fun(Row) ->
            DocId = couchbeam_doc:get_value(<<"id">>, Row),
            maps:get(DocId, TagCounts) =:= length(Tags)
        end, AllRows),
    % remove the duplicate rows
    %   [[{<<"id">>, <<"test_AC">>}, {<<"key">>, <<"cat">>}],
    %    [{<<"id">>, <<"test_AC">>}, {<<"key">>, <<"picnic">>}]]
    case MatchingRows of
        [] ->
            MatchingRows;
        _ ->
            lists:foldr(fun(Row, AccIn) ->
                    NewDocId = couchbeam_doc:get_value(<<"id">>, Row),
                    PrevDocId = couchbeam_doc:get_value(<<"id">>, hd(AccIn)),
                    if NewDocId =:= PrevDocId ->
                        AccIn;
                       true ->
                        [Row|AccIn]
                    end
                end, [hd(MatchingRows)], tl(MatchingRows))
    end.

%
% @doc Retrieves all documents whose most relevant date is within the given year.
%      The date used will be exif_date, or file_date, or import_date, in that
%      order. Results are as from couchbeam_view:fetch/3.
%
-spec by_date(Year) -> Rows
    when Year :: integer(),
         Rows :: [term()].
by_date(Year) when is_integer(Year) ->
    StartDate = [Year, 0, 0, 0, 0],
    EndDate = [Year + 1, 0, 0, 0, 0],
    gen_server:call(tanuki_backend_db, {by_date, StartDate, EndDate}).

%
% @doc Retrieves all documents whose most relevant date is within the given month.
%      The date used will be exif_date, or file_date, or import_date, in that
%      order. Results are as from couchbeam_view:fetch/3.
%
-spec by_date(Year, Month) -> Rows
    when Year  :: integer(),
         Month :: integer(),
         Rows  :: [term()].
by_date(Year, Month) when is_integer(Year), is_integer(Month) ->
    StartDate = [Year, Month, 0, 0, 0],
    EndDate = [Year, Month + 1, 0, 0, 0],
    gen_server:call(tanuki_backend_db, {by_date, StartDate, EndDate}).

%
% @doc Converts a date-list (list of integers representing a date) of the
%      form [2014, 7, 4, 12, 1] to a string: 2014/7/4 12:01.
%
-spec date_list_to_string(DateList) -> DateString
    when DateList   :: [integer()],
         DateString :: string().
date_list_to_string(DateList) ->
    lists:flatten(io_lib:format("~B/~B/~B ~B:~B", DateList)).

%
% @doc Converts a date-list (list of integers representing a date) of the
%      form [2014, 7, 4, 12, 1] to a string, with only the date: 2014/7/4.
%
-spec date_list_to_string(DateList, date_only) -> DateString
    when DateList   :: [integer()],
         DateString :: string().
date_list_to_string(DateList, date_only) ->
    lists:flatten(io_lib:format("~B/~B/~B", lists:sublist(DateList, 3))).

%
% @doc Retrieves the mimetype for a document with the given checksum, in
%      a form suitable for the Cowboy dispatch mimetype handler.
%
-spec path_to_mimes(Path) -> MimeType
    when Path     :: binary(),
         MimeType :: {binary(), binary(), []}.
path_to_mimes(Path) when is_binary(Path) ->
    Path2 = binary_to_list(Path),
    Parts = string:tokens(Path2, "/"),
    Checksum = string:join(lists:sublist(Parts, length(Parts) - 2, 3), ""),
    case by_checksum(Checksum) of
        [] -> {<<"application">>, <<"octet-stream">>, []};
        [Doc|_T] ->
            DocValue = couchbeam_doc:get_value(<<"value">>, Doc),
            [Type, SubType] = binary:split(DocValue, <<"/">>),
            {Type, SubType, []}
    end.

%
% @doc Returns an ETag for a given file, which essentially means converting
%      the file path to a sha256 checksum.
%
-spec generate_etag(Path, Size, Mtime) -> {strong, Etag}
    when Path  :: binary(),
         Size  :: integer(),
         Mtime :: term(),
         Etag  :: binary().
generate_etag(Path, _Size, _Mtime) ->
    Filepath = binary_to_list(Path),
    Parts = string:tokens(Filepath, "/"),
    Checksum = string:join(lists:sublist(Parts, length(Parts) - 2, 3), ""),
    {strong, list_to_binary(Checksum)}.

%
% @doc Either retrieve the thumbnail produced earlier, or generate one
%      now and cache for later use. Returns {ok, Binary, Mimetype}.
%
-spec retrieve_thumbnail(Checksum, RelativePath) -> {ok, Image, MimeType}
    when Checksum     :: string(),
         RelativePath :: string(),
         Image        :: bitstring(),
         MimeType     :: binary().
retrieve_thumbnail(Checksum, RelativePath) ->
    % look for thumbnail cached in mnesia, producing and storing, if needed
    F = fun() ->
        case mnesia:read(thumbnails, Checksum) of
            [#thumbnails{sha256=_C, binary=Binary}] ->
                % record the time this thumbnail was accessed
                T = seconds_since_epoch(),
                mnesia:write(#thumbnail_dates{timestamp=T, sha256=Checksum}),
                Binary;
            [] ->
                % producing a thumbnail in a transaction is not ideal...
                Binary = generate_thumbnail(RelativePath),
                mnesia:write(#thumbnails{sha256=Checksum, binary=Binary}),
                T = seconds_since_epoch(),
                mnesia:write(#thumbnail_dates{timestamp=T, sha256=Checksum}),
                % update the count of thumbnails currently cached
                Count = mnesia:dirty_update_counter(thumbnail_counter, id, 1),
                % prune oldest record if we reached our limit
                if Count > 1000 ->
                    % this finds the oldest thumbnail, where age is determined by
                    % either when it was generated or when it was last retrieved
                    OldestKey = mnesia:first(thumbnail_dates),
                    [#thumbnail_dates{sha256=OC}] = mnesia:read(thumbnail_dates, OldestKey),
                    mnesia:delete({thumbnails, OC}),
                    mnesia:delete({thumbnail_dates, OldestKey}),
                    mnesia:dirty_update_counter(thumbnail_counter, id, -1);
                   true -> ok
                end,
                Binary
        end
    end,
    Binary = mnesia:activity(transaction, F),
    % thumbnails are always jpeg
    {ok, Binary, <<"image/jpeg">>}.

%
% @doc Produce a jpeg thumbnail of the named image file.
%
-spec generate_thumbnail(RelativePath) -> Image
    when RelativePath :: string(),
         Image        :: binary().
generate_thumbnail(RelativePath) ->
    {ok, AssetsDir} = application:get_env(tanuki_backend, assets_dir),
    SourceFile = filename:join(AssetsDir, RelativePath),
    {ok, ImageData} = file:read_file(SourceFile),
    emagick_rs:image_fit(ImageData, 240, 240).

%
% @doc Return the seconds since the epoch (1970/1/1 00:00).
%
-spec seconds_since_epoch() -> integer().
seconds_since_epoch() ->
    {Mega, Sec, _Micro} = os:timestamp(),
    Mega * 1000000 + Sec.

%
% @doc Extract the most accurate date from the given document. The precedence
%      is EXIF original date, followed by file date, followed by import date.
%      The date is the format stored in the database (a list of integers).
%
-spec get_best_date(Doc) -> Result
    when Doc    :: term(),
         Result :: list() | none.
get_best_date(Doc) ->
    case get_field_value(<<"exif_date">>, Doc) of
        none ->
            case get_field_value(<<"file_date">>, Doc) of
                none ->
                    get_field_value(<<"import_date">>, Doc);
                Date -> Date
            end;
        Date -> Date
    end.

%
% @doc Extract the value of the named field, or none if either undefined or null.
%
-spec get_field_value(Field, Document) -> Result
    when Field    :: term(),
         Document :: binary(),
         Result   :: term() | none.
get_field_value(Field, Document) ->
    case couchbeam_doc:get_value(Field, Document) of
        undefined ->
            none;
        null ->
            none;
        Value ->
            Value
    end.
