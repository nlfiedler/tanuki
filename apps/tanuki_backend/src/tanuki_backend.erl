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

-export([by_checksum/1, by_date/1, by_date/2, by_tags/1, by_tags/2]).
-export([all_tags/0, fetch_document/1, update_document/1]).
-export([get_best_date/1, date_list_to_string/1, date_list_to_string/2]).
-export([retrieve_thumbnail/1, get_field_value/2, seconds_since_epoch/0]).
-export([checksum_to_asset_path/1, generate_thumbnail/2]).

-include_lib("kernel/include/file.hrl").
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

% @doc
%
% Update the given document in the database.
%
-spec update_document(Doc) -> {ok, NewDoc} | {error, Reason}
    when Doc    :: term(),
         NewDoc :: term(),
         Reason :: term().
update_document(Doc) ->
    gen_server:call(tanuki_backend_db, {update_document, Doc}).

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

% @doc
%
% Retrieves all documents with the given tags, as couchbeam view results.
% Only those documents containing all of the given tags will be returned.
% Ordering is non-deterministic.
%
% Results are cached based on the tags, so any changes in the database will
% not be reflected by repeated queries for the same set of tags. However,
% this serves repeated requests very well, such as for allowing efficient
% pagination of many results.
%
-spec by_tags(Tags) -> Rows
    when Tags :: [string()],
         Rows :: [term()].
by_tags(Tags) when is_list(Tags) ->
    by_tags(Tags, fun(_A, _B) -> true end).

% @doc
%
% Like by_tags/1 with the results sorted according to the given function.
%
-spec by_tags(Tags, SortFun) -> Rows
    when Tags    :: [string()],
         SortFun :: fun((term(), term()) -> boolean()),
         Rows    :: [term()].
by_tags(Tags, SortFun) when is_list(Tags) ->
    % Ensure the results cache table exists for this process.
    case ets:info(by_tags) of
        undefined ->
            lager:info("creating by_tags ETS table"),
            ets:new(by_tags, [set, named_table, compressed]);
        _InfoList -> ok
    end,
    % Produce a key based on the requested tags and the sorting function
    % such that if either change, the key changes.
    Key = {list_to_binary(string:join(lists:sort(Tags), "")), SortFun},
    case ets:lookup(by_tags, Key) of
        [] ->
            lager:info("cache miss for ~w", [Key]),
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
            % Remove the duplicate rows by sorting on the document
            % identifier in a unique fashion.
            UnsortedResults = lists:usort(fun(A, B) ->
                    IdA = couchbeam_doc:get_value(<<"id">>, A),
                    IdB = couchbeam_doc:get_value(<<"id">>, B),
                    IdA =< IdB
                end, MatchingRows),
            % Now sort the rows according to the given sorting function.
            Results = lists:sort(SortFun, UnsortedResults),
            % Save the results in ETS to avoid doing the same work again if
            % a repeated request is made, replacing any previous values.
            ets:delete_all_objects(by_tags),
            ets:insert(by_tags, {Key, Results}),
            Results;
        [{_Key, Rows}] ->
            lager:info("cache hit for ~w", [Key]),
            Rows
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

% @doc
%
% For a given SHA256 checksum, return the path to the asset.
%
-spec checksum_to_asset_path(Checksum) -> Filepath
    when Checksum :: string(),
         Filepath :: string().
checksum_to_asset_path(Checksum) ->
    {ok, AssetsDir} = application:get_env(tanuki_backend, assets_dir),
    Part1 = string:sub_string(Checksum, 1, 2),
    Part2 = string:sub_string(Checksum, 3, 4),
    Part3 = string:sub_string(Checksum, 5),
    RelativePath = filename:join([Part1, Part2, Part3]),
    filename:join(AssetsDir, RelativePath).

%
% @doc Either retrieve the thumbnail produced earlier, or generate one
%      now and cache for later use. Returns {ok, Binary, Mimetype}.
%
-spec retrieve_thumbnail(Checksum) -> {ok, Image, MimeType}
    when Checksum     :: string(),
         Image        :: bitstring(),
         MimeType     :: binary().
retrieve_thumbnail(Checksum) ->
    % look for thumbnail cached in mnesia, producing and storing, if needed
    F = fun() ->
        case mnesia:read(thumbnails, Checksum) of
            [#thumbnails{sha256=_C, binary=Binary}] ->
                % record the time this thumbnail was accessed
                lager:info("cache hit for thumbnail ~w", [Checksum]),
                T = seconds_since_epoch(),
                ok = mnesia:write(#thumbnail_dates{timestamp=T, sha256=Checksum}),
                Binary;
            [] ->
                lager:info("cache miss for thumbnail ~w", [Checksum]),
                % producing a thumbnail in a transaction is not ideal...
                Binary = generate_thumbnail(Checksum, thumbnail),
                ok = mnesia:write(#thumbnails{sha256=Checksum, binary=Binary}),
                T = seconds_since_epoch(),
                ok = mnesia:write(#thumbnail_dates{timestamp=T, sha256=Checksum}),
                % update the count of thumbnails currently cached
                Count = mnesia:dirty_update_counter(thumbnail_counter, id, 1),
                % prune oldest record if we reached our limit
                if Count > 1000 ->
                    % this finds the oldest thumbnail, where age is determined by
                    % either when it was generated or when it was last retrieved
                    lager:info("discarding oldest cached thumbnail"),
                    OldestKey = mnesia:first(thumbnail_dates),
                    [#thumbnail_dates{sha256=OC}] = mnesia:read(thumbnail_dates, OldestKey),
                    ok = mnesia:delete({thumbnails, OC}),
                    ok = mnesia:delete({thumbnail_dates, OldestKey}),
                    mnesia:dirty_update_counter(thumbnail_counter, id, -1);
                   true -> ok
                end,
                Binary
        end
    end,
    Binary = mnesia:activity(transaction, F),
    % thumbnails are always jpeg
    {ok, Binary, <<"image/jpeg">>}.

% @doc
%
% Produce a jpeg thumbnail of the image file designated by the given SHA256
% checksum. Two convenient sizes are available, either 'thumbnail' which
% resizes the image to a box of 240 by 240 pixels, or 'preview', which
% resizes the image to a box of 640 by 640 pixels. Or you can provide an
% integer value of your own choosing.
%
-spec generate_thumbnail(Checksum, Size) -> Image
    when Checksum :: string(),
         Size     :: thumbnail | preview | integer(),
         Image    :: binary().
generate_thumbnail(Checksum, thumbnail) ->
    generate_thumbnail(Checksum, 240);
generate_thumbnail(Checksum, preview) ->
    generate_thumbnail(Checksum, 640);
generate_thumbnail(Checksum, Size) when is_integer(Size) ->
    SourceFile = checksum_to_asset_path(Checksum),
    % Avoid attempting to generate a thumbnail for large files, which are
    % very likely not image files at all (e.g. videos). The value of 10MB
    % was arrived at by examining a collection of images and videos that
    % represent typical usage. That is, most images are less than 10MB and
    % most videos are over 10MB; the overlap is acceptable, such that some
    % large images will not get thumbnails, and some videos will be pulled
    % into memory only to result in an error.
    case file:read_file_info(SourceFile) of
        {ok, #file_info{size = Bytes}} when Bytes < 10*1048576 ->
            {ok, ImageData} = file:read_file(SourceFile),
            case emagick_rs:image_fit(ImageData, Size, Size) of
                {ok, Resized} -> Resized;
                {error, Reason} ->
                    lager:warning("unable to resize asset ~s: ~p", [Checksum, Reason]),
                    broken_image_placeholder()
            end;
        _ -> broken_image_placeholder()
    end.

%
% @doc Return the image data for the broken image placeholder thumbnail.
%
broken_image_placeholder() ->
    PrivPath = code:priv_dir(tanuki_backend),
    ImagePath = filename:join(PrivPath, "images/broken_image.jpg"),
    {ok, BrokenData} = file:read_file(ImagePath),
    BrokenData.

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
