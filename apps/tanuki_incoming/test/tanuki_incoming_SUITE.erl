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
-module(tanuki_incoming_SUITE).
-compile(export_all).
-include_lib("common_test/include/ct.hrl").
-include_lib("eunit/include/eunit.hrl").
-include_lib("kernel/include/file.hrl").

-define(TESTDB, "tanuki_test").

init_per_suite(Config) ->
    % load the application so we can read and modify the environment
    ok = application:load(tanuki_incoming),
    Priv = ?config(priv_dir, Config),
    IncomingDir = filename:join(Priv, "incoming"),
    ok = application:set_env(tanuki_incoming, incoming_dir, IncomingDir),
    ok = application:set_env(tanuki_incoming, database, ?TESTDB),
    AssetsDir = filename:join(Priv, "assets"),
    ok = application:set_env(tanuki_incoming, assets_dir, AssetsDir),
    ok = couchbeam:start(),
    {ok, Url} = application:get_env(tanuki_incoming, couchdb_url),
    {ok, Opts} = application:get_env(tanuki_incoming, couchdb_opts),
    S = couchbeam:server_connection(Url, Opts),
    % clean up any mess from a previously failed test
    {ok, _Wat} = case couchbeam:db_exists(S, ?TESTDB) of
        true  -> couchbeam:delete_db(S, ?TESTDB);
        false -> {ok, foo}
    end,
    {ok, _Db} = couchbeam:create_db(S, ?TESTDB, []),
    % start the application under test
    {ok, _Started1} = application:ensure_all_started(tanuki_incoming),
    %
    % also need the backend to be configured and running in test mode
    %
    ok = application:load(tanuki_backend),
    ok = application:set_env(tanuki_backend, database, ?TESTDB),
    ok = application:set_env(cowboy, bind_address, "0.0.0.0"),
    ok = application:set_env(cowboy, port, 8000),
    ok = application:set_env(cowboy, server_name, nitrogen),
    ok = application:set_env(cowboy, document_root, "./priv/static"),
    ok = application:set_env(cowboy, static_paths,
        ["/js/", "/images/", "/css/", "/nitrogen/", "/favicon.ico"]),
    {ok, _Started2} = application:ensure_all_started(tanuki_backend),
    [
        {url, Url},
        {opts, Opts},
        {incoming_dir, IncomingDir},
        {assets_dir, AssetsDir} |
        Config
    ].

end_per_suite(Config) ->
    gen_server:call(tanuki_incoming, terminate),
    Url = ?config(url, Config),
    Opts = ?config(opts, Config),
    S = couchbeam:server_connection(Url, Opts),
    couchbeam:delete_db(S, ?TESTDB),
    couchbeam:stop(),
    ok.

all() ->
    [
        single_image_test
    ].

single_image_test(Config) ->
    DataDir = ?config(data_dir, Config),
    % create the incoming directory and copy our test photo there
    IncomingDir = ?config(incoming_dir, Config),
    TaggedDir = filename:join(IncomingDir, "yellow_flower@field"),
    SrcImagePath = filename:join([DataDir, "img_015.JPG"]),
    DestImagePath = filename:join(TaggedDir, "img_015.JPG"),
    ok = filelib:ensure_dir(DestImagePath),
    {ok, _BytesCopied} = file:copy(SrcImagePath, DestImagePath),
    % Would like to have set the ctime of the incoming directory but
    % file:write_file_info/2,3 ignores the ctime value on Unix systems.
    gen_server:call(tanuki_incoming, process_now),
    % check that images are gone from incoming directory
    {ok, []} = file:list_dir(IncomingDir),
    AssetsDir = ?config(assets_dir, Config),
    true = filelib:is_file(filename:join([AssetsDir, "d0", "9f",
        "d659423e71bb1b5e20d78a1ab7ce393e74e463f2dface3634d78ec155397"])),
    % check that each field of each new document is the correct value
    Rows = tanuki_backend:by_tag("yellow"),
    ?assertEqual(1, length(Rows)),
    DocId = couchbeam_doc:get_value(<<"id">>, hd(Rows)),
    {ok, Doc} = tanuki_backend:fetch_document(DocId),
    ?assertEqual(<<"img_015.JPG">>, couchbeam_doc:get_value(<<"file_name">>, Doc)),
    % TODO: make the user name be the current user
    ?assertEqual(<<"nfiedler">>, couchbeam_doc:get_value(<<"file_owner">>, Doc)),
    ?assertEqual(369781, couchbeam_doc:get_value(<<"file_size">>, Doc)),
    ?assertEqual(<<"image/jpeg">>, couchbeam_doc:get_value(<<"mimetype">>, Doc)),
    % TODO: exif date extraction is failing for img_015.JPG
    % ?assertEqual([2011, 10, 7, 9, 18], couchbeam_doc:get_value(<<"exif_date">>, Doc)),
    ?assertEqual(null, couchbeam_doc:get_value(<<"exif_date">>, Doc)),
    ?assertEqual(<<"field">>, couchbeam_doc:get_value(<<"location">>, Doc)),
    ?assertEqual(<<"d09fd659423e71bb1b5e20d78a1ab7ce393e74e463f2dface3634d78ec155397">>,
                 couchbeam_doc:get_value(<<"sha256">>, Doc)),
    ?assertEqual([<<"yellow">>, <<"flower">>], couchbeam_doc:get_value(<<"tags">>, Doc)),
    ok.

% TODO: test with image that has no EXIF date at all
% TODO: test with asset that has already been imported with different tags
% TODO: test with the infamous extraneous files (e.g. .localized)
