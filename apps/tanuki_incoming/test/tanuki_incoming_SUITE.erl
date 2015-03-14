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

-define(TESTDB, "tanuki_test").

init_per_suite(Config) ->
    % load the application so we can read and modify the environment
    ok = application:load(tanuki_incoming),
    % TODO: configure the 'incoming_dir' env var
    ok = application:set_env(tanuki_incoming, database, ?TESTDB),
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
    % start the application(s)
    {ok, _Started} = application:ensure_all_started(tanuki_incoming),
    [{url, Url}, {opts, Opts} | Config].

end_per_suite(Config) ->
    gen_server:call(tanuki_incoming, terminate),
    Url = ?config(url, Config),
    Opts = ?config(opts, Config),
    S = couchbeam:server_connection(Url, Opts),
    couchbeam:delete_db(S, ?TESTDB),
    couchbeam:stop(),
    application:stop(mnesia),
    ok.

all() ->
    [
        sample_test
    ].

sample_test(_Config) ->
    ok.

% TODO: will need to install the views to get by_checksum view (for testing)
% TODO: copy test images into the 'incoming' directory
% TODO: invoke tanuki_incoming:process_path/3
% TODO: check that images are gone from incoming directory
% TODO: check that each field of each new document is the correct value
