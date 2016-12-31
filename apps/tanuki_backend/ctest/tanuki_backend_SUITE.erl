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
-module(tanuki_backend_SUITE).
-compile(export_all).
-include_lib("common_test/include/ct.hrl").
-include_lib("eunit/include/eunit.hrl").

init_per_suite(Config) ->
    % load the application so we can read and modify the environment
    ok = application:load(tanuki_backend),
    {ok, Database} = application:get_env(tanuki_backend, database),
    Priv = ?config(priv_dir, Config),
    AssetsDir = filename:join(Priv, "assets"),
    ok = application:set_env(tanuki_backend, assets_dir, AssetsDir),
    {ok, _Started1} = application:ensure_all_started(couchbeam),
    {ok, Url} = application:get_env(tanuki_backend, couchdb_url),
    {ok, Opts} = application:get_env(tanuki_backend, couchdb_opts),
    S = couchbeam:server_connection(Url, Opts),
    % clean up any mess from a previously failed test
    {ok, _Wat} = case couchbeam:db_exists(S, Database) of
        true  -> couchbeam:delete_db(S, Database);
        false -> {ok, foo}
    end,
    {ok, Db} = couchbeam:create_db(S, Database, []),
    add_test_docs(Db, Config),
    % set up mnesia
    ok = application:set_env(mnesia, dir, Priv),
    tanuki_backend_app:ensure_schema([node()]),
    ok = application:start(mnesia),
    % start the application(s)
    {ok, _Started2} = application:ensure_all_started(tanuki_backend),
    [
        {url, Url},
        {opts, Opts},
        {assets_dir, AssetsDir} |
        Config
    ].

get_data_dir(Config) ->
    % The ct task from mix_erlang_tasks plugin does not set the data_dir
    % and Common Test defaults to where the beam file is located, which is
    % not where the data directory happens to be.
    DataDir = ?config(data_dir, Config),
    case filelib:is_dir(DataDir) of
        false ->
            SourcePath = proplists:get_value(source, tanuki_backend_SUITE:module_info(compile)),
            filename:join(filename:dirname(SourcePath), "tanuki_backend_SUITE_data");
        true ->
            DataDir
    end.

add_test_docs(Db, Config) ->
    % populate test database from json files in data_dir
    DataDir = get_data_dir(Config),
    InsertDocument = fun(Filename) ->
        Filepath = filename:join([DataDir, Filename]),
        {ok, Binary} = file:read_file(Filepath),
        Json = couchbeam_ejson:decode(Binary),
        {ok, _Doc1} = couchbeam:save_doc(Db, Json)
    end,
    {ok, Filenames} = file:list_dir(DataDir),
    JsonSelector = fun(Name) -> filename:extension(Name) == ".json" end,
    JsonFiles = lists:filter(JsonSelector, Filenames),
    [InsertDocument(Filename) || Filename <- JsonFiles],
    ok.

end_per_suite(Config) ->
    application:stop(tanuki_backend_db),
    Url = ?config(url, Config),
    Opts = ?config(opts, Config),
    S = couchbeam:server_connection(Url, Opts),
    {ok, Database} = application:get_env(tanuki_backend, database),
    couchbeam:delete_db(S, Database),
    application:stop(couchbeam),
    application:stop(mnesia),
    ok.

all() ->
    [
        fetch_document,
        all_tags,
        by_checksum,
        by_tag,
        by_tags,
        by_year,
        by_month,
        date_formatter,
        path_to_mimes,
        generate_etag,
        get_best_date,
        generate_thumbnail
    ].

fetch_document(_Config) ->
    {ok, Document1} = tanuki_backend:fetch_document("test_AA"),
    Content1 = couchbeam_doc:get_value(<<"file_owner">>, Document1),
    ?assertEqual(<<"akwok">>, Content1),
    % negative case, not found
    ?assertEqual({error, not_found}, tanuki_backend:fetch_document("foobar")),
    ok.

all_tags(_Config) ->
    Rows = tanuki_backend:all_tags(),
    ?assertEqual(6, length(Rows)),
    Validate = fun(Row, Key, Value) ->
        ?assertEqual(Key, couchbeam_doc:get_value(<<"key">>, Row)),
        ?assertEqual(Value, couchbeam_doc:get_value(<<"value">>, Row))
    end,
    Keys = [<<"cat">>, <<"cheeseburger">>, <<"christina">>, <<"dog">>, <<"joseph">>, <<"picnic">>],
    Values = [2, 1, 3, 1, 3, 2],
    Expected = lists:zip3(Rows, Keys, Values),
    [Validate(Row, Key, Value) || {Row, Key, Value} <- Expected],
    ok.

by_checksum(_Config) ->
    Checksum = "39092991d6dde424191780ea7eac2f323accc5686075e3150cbb8fc5da331100",
    Rows = tanuki_backend:by_checksum(Checksum),
    ?assertEqual(1, length(Rows)),
    Validate = fun(Row, Id, Value) ->
        % view has "id" but documents have "_id"? weird
        ?assertEqual(Id, couchbeam_doc:get_value(<<"id">>, Row)),
        ?assertEqual(Value, couchbeam_doc:get_value(<<"value">>, Row))
    end,
    Ids = [<<"test_AA">>],
    Values = [<<"image/jpeg">>],
    [Validate(Row, Id, Value) || {Row, Id, Value} <- lists:zip3(Rows, Ids, Values)],
    % negative case, no matching checksum
    ?assertEqual([], tanuki_backend:by_checksum("foobar")),
    ok.

by_tag(_Config) ->
    Rows = tanuki_backend:by_tag("cat"),
    ?assertEqual(2, length(Rows)),
    Validate = fun(Row, Id) ->
        % view has "id" but documents have "_id"? weird
        ?assertEqual(Id, couchbeam_doc:get_value(<<"id">>, Row))
    end,
    Keys = [<<"test_AA">>, <<"test_AC">>],
    [Validate(Row, Key) || {Row, Key} <- lists:zip(Rows, Keys)],
    % negative case, no such tag
    ?assertEqual([], tanuki_backend:by_tag("foobar")),
    ok.

by_tags(_Config) ->
    Rows = tanuki_backend:by_tags(["christina", "joseph"]),
    ?assertEqual(3, length(Rows)),
    Validate = fun(Row, Id) ->
        % view has "id" but documents have "_id"? weird
        ?assertEqual(Id, couchbeam_doc:get_value(<<"id">>, Row))
    end,
    Ids = [<<"test_AD">>, <<"test_AE">>, <<"test_AF">>],
    [Validate(Row, Id) || {Row, Id} <- lists:zip(Rows, Ids)],
    % negative case, no such tags
    ?assertEqual([], tanuki_backend:by_tags(["foo", "bar"])),
    ok.

by_year(_Config) ->
    Validate = fun(Input, Count, ExpectedIds) ->
        Rows = tanuki_backend:by_date(Input),
        ?assertEqual(Count, length(Rows)),
        [?assertEqual(Id, couchbeam_doc:get_value(<<"id">>, Row)) ||
            {Id, Row} <- lists:zip(ExpectedIds, Rows) ]
    end,
    Inputs = [2013, 2014, 2015],
    Counts = [1, 2, 3],
    Ids = [
        [<<"test_AA">>],
        [<<"test_AC">>, <<"test_AB">>],
        [<<"test_AD">>, <<"test_AE">>, <<"test_AF">>]
    ],
    [Validate(I, C, E) || {I, C, E} <- lists:zip3(Inputs, Counts, Ids)],
    ok.

by_month(_Config) ->
    Validate = fun({Year, Month}, Count, ExpectedIds) ->
        Rows = tanuki_backend:by_date(Year, Month),
        ?assertEqual(Count, length(Rows)),
        [?assertEqual(Id, couchbeam_doc:get_value(<<"id">>, Row)) ||
            {Id, Row} <- lists:zip(ExpectedIds, Rows) ]
    end,
    Inputs = [{2014, 7}, {2014, 10}, {2015, 4}],
    Counts = [1, 1, 3],
    Ids = [
        [<<"test_AC">>],
        [<<"test_AB">>],
        [<<"test_AD">>, <<"test_AE">>, <<"test_AF">>]
    ],
    [Validate(I, C, E) || {I, C, E} <- lists:zip3(Inputs, Counts, Ids)],
    ok.

date_formatter(_Config) ->
    Result1 = tanuki_backend:date_list_to_string([2014, 12, 23, 22, 28]),
    ?assertEqual("2014/12/23 22:28", Result1),
    Result2 = tanuki_backend:date_list_to_string([2014, 12, 23, 22, 28], date_only),
    ?assertEqual("2014/12/23", Result2),
    ok.

path_to_mimes(_Config) ->
    % success case leading path
    Filepath1 = <<"foo/39/09/2991d6dde424191780ea7eac2f323accc5686075e3150cbb8fc5da331100">>,
    Result1 = tanuki_backend:path_to_mimes(Filepath1),
    ?assertEqual({<<"image">>, <<"jpeg">>, []}, Result1),
    % success case exact path
    Filepath2 = <<"39/09/2991d6dde424191780ea7eac2f323accc5686075e3150cbb8fc5da331100">>,
    Result2 = tanuki_backend:path_to_mimes(Filepath2),
    ?assertEqual({<<"image">>, <<"jpeg">>, []}, Result2),
    % success case binary path
    Filepath3 = <<"39/09/2991d6dde424191780ea7eac2f323accc5686075e3150cbb8fc5da331100">>,
    Result3 = tanuki_backend:path_to_mimes(Filepath3),
    ?assertEqual({<<"image">>, <<"jpeg">>, []}, Result3),
    % unknown checksum case
    Filepath4 = <<"foo/11/22/34567890abcdef">>,
    Result4 = tanuki_backend:path_to_mimes(Filepath4),
    ?assertEqual({<<"application">>, <<"octet-stream">>, []}, Result4),
    % fail case no slashes
    ?assertError(function_clause, tanuki_backend:path_to_mimes(<<"abcdef">>)),
    % fail case short path
    ?assertError(function_clause, tanuki_backend:path_to_mimes(<<"abc/def">>)),
    ok.

generate_etag(_Config) ->
    % success case leading path
    Input1 = <<"foo/39/09/2991d6dde424191780ea7eac2f323accc5686075e3150cbb8fc5da331100">>,
    {strong, Result1} = tanuki_backend:generate_etag(Input1, ignored, ignored),
    ?assertEqual(<<"39092991d6dde424191780ea7eac2f323accc5686075e3150cbb8fc5da331100">>, Result1),
    % success case exact path
    Input2 = <<"39/09/2991d6dde424191780ea7eac2f323accc5686075e3150cbb8fc5da331100">>,
    {strong, Result2} = tanuki_backend:generate_etag(Input2, ignored, ignored),
    ?assertEqual(<<"39092991d6dde424191780ea7eac2f323accc5686075e3150cbb8fc5da331100">>, Result2),
    % fail case no slashes
    ?assertError(function_clause, tanuki_backend:generate_etag(<<"abcdef">>, ignored, ignored)),
    % fail case short path
    ?assertError(function_clause, tanuki_backend:generate_etag(<<"abc/def">>, ignored, ignored)),
    ok.

get_best_date(_Config) ->
    {ok, Document1} = tanuki_backend:fetch_document("test_AA"),
    ?assertEqual([2013, 1, 31, 5, 26], tanuki_backend:get_best_date(Document1)),
    {ok, Document2} = tanuki_backend:fetch_document("test_AB"),
    ?assertEqual([2014, 10, 24, 15, 9], tanuki_backend:get_best_date(Document2)),
    {ok, Document3} = tanuki_backend:fetch_document("test_AC"),
    ?assertEqual([2014, 7, 15, 3, 13], tanuki_backend:get_best_date(Document3)),
    ok.

generate_thumbnail(Config) ->
    DataDir = get_data_dir(Config),
    AssetsDir = ?config(assets_dir, Config),
    Checksum = "8dcaf3d73a10548e445ebb27ff34d7159cc5d7f3730c24e95ffe5b07bd2300fd",
    RelativePath = "8d/ca/f3d73a10548e445ebb27ff34d7159cc5d7f3730c24e95ffe5b07bd2300fd",
    SrcImagePath = filename:join(DataDir, "fighting_kittens.jpg"),
    DestImagePath = filename:join(AssetsDir, RelativePath),
    ok = filelib:ensure_dir(DestImagePath),
    {ok, _BytesCopied} = file:copy(SrcImagePath, DestImagePath),
    {ok, Binary, Mimetype} = tanuki_backend:retrieve_thumbnail(Checksum, RelativePath),
    ?assertEqual(<<"image/jpeg">>, Mimetype),
    ?assert(is_binary(Binary)),
    % TODO: Ideally would verify the image dimensions, rather than trying
    %       to test the file size, which varies slightly from OS to OS.
    ?assert(length(binary_to_list(Binary)) < 14000),
    ok.
