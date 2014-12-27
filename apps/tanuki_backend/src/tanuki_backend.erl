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
-export([all_tags/0, date_list_to_string/1, fetch_document/1, path_to_mimes/2]).
-export([generate_etag/2, produce_thumbnail/2]).

%%
%% Client API
%%

%
% @doc Retrieves the document with the given identifier as an ejson object
%      (where ejson is defined in couchbeam as essentially a proplist of
%      binary strings).
%
-spec fetch_document(string()) -> {document, term()}.
fetch_document(DocId) ->
    gen_server:call(tanuki_backend_db, {fetch_document, DocId}).

%
% @doc Retrieves all known tags as couchbeam view results.
%
-spec all_tags() -> [Rows::term()].
all_tags() ->
    gen_server:call(tanuki_backend_db, all_tags).

%
% @doc Retrieves all documents with a given checksum, as couchbeam view results.
%      Result includes the id and mimetype fields.
%
-spec by_checksum(string()) -> [Rows::term()].
by_checksum(Checksum) when is_list(Checksum) ->
    gen_server:call(tanuki_backend_db, {by_checksum, Checksum}).

%
% @doc Retrieves all documents with a given tag, as couchbeam view results.
%
-spec by_tag(string()) -> [Rows::term()].
by_tag(Tag) when is_list(Tag) ->
    gen_server:call(tanuki_backend_db, {by_tag, Tag}).

%
% @doc Retrieves all documents with the given tags, as couchbeam view results.
%
-spec by_tags([string()]) -> [Rows::term()].
by_tags(Tags) when is_list(Tags) ->
    gen_server:call(tanuki_backend_db, {by_tags, Tags}).

%
% @doc Retrieves all documents whose most relevant date is within the given year.
%      The date used will be exif_date, or file_date, or import_date, in that
%      order. Results are as from couchbeam_view:fetch/3.
%
-spec by_date(integer()) -> [Rows::term()].
by_date(Year) when is_integer(Year) ->
    StartDate = [Year, 0, 0, 0, 0],
    EndDate = [Year + 1, 0, 0, 0, 0],
    gen_server:call(tanuki_backend_db, {by_date, StartDate, EndDate}).

%
% @doc Retrieves all documents whose most relevant date is within the given month.
%      The date used will be exif_date, or file_date, or import_date, in that
%      order. Results are as from couchbeam_view:fetch/3.
%
-spec by_date(integer(), integer()) -> [Rows::term()].
by_date(Year, Month) when is_integer(Year), is_integer(Month) ->
    StartDate = [Year, Month, 0, 0, 0],
    EndDate = [Year, Month + 1, 0, 0, 0],
    gen_server:call(tanuki_backend_db, {by_date, StartDate, EndDate}).

%
% @doc Converts a date-list (list of integers representing a date) of the
%      form [2014, 7, 4, 12, 1] and converts it to a string: 2014/7/4 12:01.
%
-spec date_list_to_string([integer()]) -> string().
date_list_to_string(Datelist) ->
    lists:flatten(io_lib:format("~B/~B/~B ~B:~B", Datelist)).

%
% @doc Retrieves the mimetype for a document with the given checksum, in
%      a form suitable for the Cowboy dispatch mimetype handler.
%
-spec path_to_mimes(string(), term()) -> {bitstring(), bitstring(), list()}.
path_to_mimes(Filename, _Database) ->
    Parts = string:tokens(Filename, "/"),
    Checksum = string:join(lists:sublist(Parts, length(Parts) - 3, 3), ""),
    case by_checksum(Checksum) of
        [] -> [<<"application/octet-stream">>];
        [H|_T] -> [couchbeam_doc:get_value(<<"value">>, H)]
    end.

%
% @doc Returns an ETag for a given file, which essentially means converting
%      the file path to a sha256 checksum.
%
generate_etag(Arguments, strong_etag_extra) ->
    {_, Filepath} = lists:keyfind(filepath, 1, Arguments),
    Parts = string:tokens(Filepath, "/"),
    Checksum = string:join(lists:sublist(Parts, length(Parts) - 3, 3), ""),
    {strong, list_to_binary(Checksum)}.

%
% @doc TODO: write doc comment
% TODO: add a -spec
%
produce_thumbnail(_Checksum, RelativePath) ->
    % TODO: check if Checksum exists in mnesia thumbnails table
    % F = fun() ->
    %     case mnesia:read({thumbnails, Checksum}) of
    %         [#thumbnails{sha256=C, binary=B}] ->
    %             B;
    %         [] ->
    %             undefined
    %     end
    % end,
    % mnesia:activity(transaction, F).
    % TODO: if yes, return data
    % TODO: if no, retrieve full image and produce thumbnail
    % TODO: store thumbnail in mnesia thumbnails table
    {ok, AssetsDir} = application:get_env(tanuki_backend, assets_dir),
    SourceFile = filename:join(AssetsDir, RelativePath),
    {ok, Binary} = file:read_file(SourceFile),
    {ok, Image} = eim:load(Binary),
    DerivedBinary = eim:derive(Image, jpg, {scale, width, 240}),
    % F = fun() ->
    %     mnesia:write(#thumbnails{sha256=Checksum, binary=DerivedBinary})
    % end,
    % mnesia:activity(transaction, F).
    {ok, DerivedBinary, <<"image/jpeg">>}.
