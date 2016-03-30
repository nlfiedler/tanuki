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
    {ok, _Started0} = application:ensure_all_started(couchbeam),
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
    ok = application:set_env(lager, lager_common_test_backend, debug),
    {ok, _Started1} = application:ensure_all_started(tanuki_incoming),
    %
    % Also need the backend to be configured and running in test mode (make
    % sure it is not already running from another test suite).
    %
    application:stop(tanuki_backend),
    application:unload(tanuki_backend),
    ok = application:load(tanuki_backend),
    ok = application:set_env(tanuki_backend, database, ?TESTDB),
    ok = application:set_env(simple_bridge, handler, nitrogen),
    ok = application:set_env(simple_bridge, backend, cowboy),
    ok = application:set_env(simple_bridge, address, "0.0.0.0"),
    ok = application:set_env(simple_bridge, port, 8000),
    ok = application:set_env(simple_bridge, document_root, "./priv/static"),
    ok = application:set_env(simple_bridge, static_paths,
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
    application:stop(tanuki_backend),
    application:stop(tanuki_incoming),
    Url = ?config(url, Config),
    Opts = ?config(opts, Config),
    S = couchbeam:server_connection(Url, Opts),
    couchbeam:delete_db(S, ?TESTDB),
    application:stop(couchbeam),
    ok.

all() ->
    [
        single_image_test,
        rotated_image_test,
        topical_image_test,
        multiple_image_test,
        empty_folder_test
    ].

%% Test importing a single image.
single_image_test(Config) ->
    DataDir = ?config(data_dir, Config),
    % create the incoming directory and copy our test photo there
    IncomingDir = ?config(incoming_dir, Config),
    TaggedDir = filename:join(IncomingDir, "yellow_flower@field"),
    SrcImagePath = filename:join(DataDir, "img_015.JPG"),
    DestImagePath = filename:join(TaggedDir, "img_015.JPG"),
    ok = filelib:ensure_dir(DestImagePath),
    {ok, 369781} = file:copy(SrcImagePath, DestImagePath),
    % Would like to have set the ctime of the incoming directory but
    % file:write_file_info/2,3 ignores the ctime value on Unix systems.
    gen_server:call(tanuki_incoming, process_now),
    % check that images are gone from incoming directory
    {ok, []} = file:list_dir(IncomingDir),
    % verify images are in the assets directory
    AssetsDir = ?config(assets_dir, Config),
    true = filelib:is_file(filename:join([AssetsDir, "3c", "e3",
        "3e2aa38db15025d74b9cb9d137a68b5640f58560fd58c1e595e9bceda4bc"])),
    % check that each field of each new document is the correct value
    Rows = tanuki_backend:by_tag("yellow"),
    ?assertEqual(1, length(Rows)),
    DocId = couchbeam_doc:get_value(<<"id">>, hd(Rows)),
    {ok, Doc} = tanuki_backend:fetch_document(DocId),
    CurrentUser = list_to_binary(os:getenv("USER")),
    ExpectedValues = #{
        <<"exif_date">>  => [2011, 10, 7, 16, 18],
        <<"file_name">>  => <<"img_015.JPG">>,
        <<"file_owner">> => CurrentUser,
        <<"file_size">>  => 326691,
        <<"location">>   => <<"field">>,
        <<"mimetype">>   => <<"image/jpeg">>,
        <<"sha256">>     => <<"3ce33e2aa38db15025d74b9cb9d137a68b5640f58560fd58c1e595e9bceda4bc">>,
        % tags are in sorted order
        <<"tags">>       => [<<"flower">>, <<"yellow">>]
    },
    maps:fold(fun(Key, Value, Elem) ->
            ?assertEqual(Value, couchbeam_doc:get_value(Key, Elem)),
            Elem
        end, Doc, ExpectedValues),
    ok.

% Test in which the incoming image has a non-optimal orientation, and it
% will be automatically corrected before storage.
rotated_image_test(Config) ->
    DataDir = ?config(data_dir, Config),
    % create the incoming directory and copy our test photo there
    IncomingDir = ?config(incoming_dir, Config),
    TaggedDir = filename:join(IncomingDir, "rotated_cats@outdoors"),
    SrcImagePath = filename:join(DataDir, "fighting_kittens.jpg"),
    DestImagePath = filename:join(TaggedDir, "fighting_kittens.jpg"),
    ok = filelib:ensure_dir(DestImagePath),
    {ok, 39932} = file:copy(SrcImagePath, DestImagePath),
    gen_server:call(tanuki_incoming, process_now),
    % check that images are gone from incoming directory
    {ok, []} = file:list_dir(IncomingDir),
    % verify images are in the assets directory
    AssetsDir = ?config(assets_dir, Config),
    true = filelib:is_file(filename:join([AssetsDir, "e9", "fb",
        "9c3c396d4ea583aa965aa85a5a228bbb591482fbe3e9b9e10820314afe1e"])),
    % check that each field of each new document is the correct value
    Rows = tanuki_backend:by_tag("rotated"),
    ?assertEqual(1, length(Rows)),
    DocId = couchbeam_doc:get_value(<<"id">>, hd(Rows)),
    {ok, Doc} = tanuki_backend:fetch_document(DocId),
    ExpectedValues = #{
        <<"exif_date">> => null,
        <<"mimetype">>  => <<"image/jpeg">>,
        <<"file_size">> => 40078,
        <<"location">>  => <<"outdoors">>,
        <<"sha256">>    => <<"e9fb9c3c396d4ea583aa965aa85a5a228bbb591482fbe3e9b9e10820314afe1e">>,
        <<"tags">>      => [<<"cats">>, <<"rotated">>]
    },
    maps:fold(fun(Key, Value, Elem) ->
            ?assertEqual(Value, couchbeam_doc:get_value(Key, Elem)),
            Elem
        end, Doc, ExpectedValues),
    ok.

%% Test importing an image with topic, tags, and location.
topical_image_test(Config) ->
    DataDir = ?config(data_dir, Config),
    % create the incoming directory and copy our test photo there
    IncomingDir = ?config(incoming_dir, Config),
    TaggedDir = filename:join(IncomingDir, "honeymoon^cows_field@hawaii"),
    SrcImagePath = filename:join(DataDir, "dcp_1069.jpg"),
    DestImagePath = filename:join(TaggedDir, "dcp_1069.jpg"),
    ok = filelib:ensure_dir(DestImagePath),
    {ok, 80977} = file:copy(SrcImagePath, DestImagePath),
    % Would like to have set the ctime of the incoming directory but
    % file:write_file_info/2,3 ignores the ctime value on Unix systems.
    gen_server:call(tanuki_incoming, process_now),
    % check that images are gone from incoming directory
    {ok, []} = file:list_dir(IncomingDir),
    % verify images are in the assets directory
    AssetsDir = ?config(assets_dir, Config),
    true = filelib:is_file(filename:join([AssetsDir, "50", "d2",
        "62207f1202e52808574fc77803801df5dcab6bd20192da2bc7d8c14d1c5a"])),
    % check that each field of each new document is the correct value
    Rows = tanuki_backend:by_tag("cows"),
    ?assertEqual(1, length(Rows)),
    DocId = couchbeam_doc:get_value(<<"id">>, hd(Rows)),
    {ok, Doc} = tanuki_backend:fetch_document(DocId),
    CurrentUser = list_to_binary(os:getenv("USER")),
    ExpectedValues = #{
        <<"exif_date">>  => [2003, 9, 3, 17, 24],
        <<"file_name">>  => <<"dcp_1069.jpg">>,
        <<"file_owner">> => CurrentUser,
        <<"file_size">>  => 81355,
        <<"location">>   => <<"hawaii">>,
        <<"mimetype">>   => <<"image/jpeg">>,
        <<"topic">>      => <<"honeymoon">>,
        <<"sha256">>     => <<"50d262207f1202e52808574fc77803801df5dcab6bd20192da2bc7d8c14d1c5a">>,
        % tags are in sorted order
        <<"tags">>       => [<<"cows">>, <<"field">>]
    },
    maps:fold(fun(Key, Value, Elem) ->
            ?assertEqual(Value, couchbeam_doc:get_value(Key, Elem)),
            Elem
        end, Doc, ExpectedValues),
    ok.

%% Test importing multiple images. This also tests updating an existing
%% asset (or two), in which the tags are merged and the location is not
%% changed.
multiple_image_test(Config) ->
    DataDir = ?config(data_dir, Config),
    % create the incoming directory and copy our test photos there
    IncomingDir = ?config(incoming_dir, Config),
    TaggedDir = filename:join(IncomingDir, "multiple@earth"),
    ok = file:make_dir(TaggedDir),
    Images = ["fighting_kittens.jpg", "img_015.JPG", "IMG_5745.JPG"],
    CopyImages = fun(Filename) ->
        SrcImagePath = filename:join(DataDir, Filename),
        DestImagePath = filename:join(TaggedDir, Filename),
        {ok, _BytesCopied} = file:copy(SrcImagePath, DestImagePath)
    end,
    ok = lists:foreach(CopyImages, Images),
    % process the incoming images now
    gen_server:call(tanuki_incoming, process_now),
    % check that images are gone from incoming directory
    {ok, []} = file:list_dir(IncomingDir),
    % verify images are in the assets directory
    AssetsDir = ?config(assets_dir, Config),
    true = filelib:is_file(filename:join([AssetsDir, "7a", "3d",
        "c0673dc24cee1224022d0cc545cef3728bd54a5b73b6b1a52a5632a1359b"])),
    true = filelib:is_file(filename:join([AssetsDir, "e9", "fb",
        "9c3c396d4ea583aa965aa85a5a228bbb591482fbe3e9b9e10820314afe1e"])),
    true = filelib:is_file(filename:join([AssetsDir, "3c", "e3",
        "3e2aa38db15025d74b9cb9d137a68b5640f58560fd58c1e595e9bceda4bc"])),
    % check specific fields of each new document
    KittensValues = #{
        <<"exif_date">> => null,
        <<"mimetype">>  => <<"image/jpeg">>,
        <<"file_size">> => 40078,
        <<"location">>  => <<"outdoors">>,
        <<"sha256">>    => <<"e9fb9c3c396d4ea583aa965aa85a5a228bbb591482fbe3e9b9e10820314afe1e">>,
        <<"tags">>      => [<<"cats">>, <<"multiple">>, <<"rotated">>]
    },
    FlowerValues = #{
        <<"exif_date">> => [2011, 10, 7, 16, 18],
        <<"mimetype">>  => <<"image/jpeg">>,
        <<"file_size">> => 326691,
        % existing location does not change
        <<"location">>  => <<"field">>,
        <<"sha256">>    => <<"3ce33e2aa38db15025d74b9cb9d137a68b5640f58560fd58c1e595e9bceda4bc">>,
        % new tags are merged with old
        <<"tags">>      => [<<"flower">>, <<"multiple">>, <<"yellow">>]
    },
    ValleyValues = #{
        <<"exif_date">> => [2014, 04, 23, 13, 33],
        <<"mimetype">>  => <<"image/jpeg">>,
        <<"file_size">> => 104061,
        <<"location">>  => <<"earth">>,
        <<"sha256">>    => <<"7a3dc0673dc24cee1224022d0cc545cef3728bd54a5b73b6b1a52a5632a1359b">>,
        <<"tags">>      => [<<"multiple">>]
    },
    FilenameToValues = #{
        <<"fighting_kittens.jpg">> => KittensValues,
        <<"img_015.JPG">> => FlowerValues,
        <<"IMG_5745.JPG">> => ValleyValues
    },
    Rows = tanuki_backend:by_tag("multiple"),
    ?assertEqual(3, length(Rows)),
    ValidateRow = fun(Row) ->
        DocId = couchbeam_doc:get_value(<<"id">>, Row),
        {ok, Doc} = tanuki_backend:fetch_document(DocId),
        Filename = couchbeam_doc:get_value(<<"file_name">>, Doc),
        ExpectedValues = maps:get(Filename, FilenameToValues),
        maps:fold(fun(Key, Value, Elem) ->
                ?assertEqual(Value, couchbeam_doc:get_value(Key, Elem)),
                Elem
            end, Doc, ExpectedValues)
    end,
    lists:foreach(ValidateRow, Rows),
    ok.

%% Test in which the incoming directory is empty, hence nothing should be imported.
%% Also tests removal of the automatically generated cruft from Mac OS X.
empty_folder_test(Config) ->
    % create an empty incoming directory
    IncomingDir = ?config(incoming_dir, Config),
    TaggedDir = filename:join(IncomingDir, "empty_folder"),
    % create some useless Mac OS X files, which will be removed
    LocalizedPath = filename:join(TaggedDir, ".localized"),
    DSStorePath = filename:join(TaggedDir, ".DS_Store"),
    ok = filelib:ensure_dir(DSStorePath),
    ok = file:write_file(LocalizedPath, <<"dummy">>),
    ok = file:write_file(DSStorePath, <<"dummy">>),
    gen_server:call(tanuki_incoming, process_now),
    % check that empty incoming directory is removed
    {ok, []} = file:list_dir(IncomingDir),
    % check that no assets were imported by those tags
    Rows = tanuki_backend:by_tag("empty"),
    ?assertEqual(0, length(Rows)),
    ok.
