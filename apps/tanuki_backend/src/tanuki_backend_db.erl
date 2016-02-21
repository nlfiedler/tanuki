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
-module(tanuki_backend_db).
-behavior(gen_server).
-export([start_link/0]).
-export([init/1, handle_call/3, handle_cast/2, handle_info/2, terminate/2, code_change/3]).

-record(state, {server, database}).

%%
%% Client API
%%
start_link() ->
    gen_server:start_link({local, ?MODULE}, ?MODULE, [], []).

%%
%% gen_server callbacks
%%
init([]) ->
    {ok, Url} = application:get_env(tanuki_backend, couchdb_url),
    {ok, Opts} = application:get_env(tanuki_backend, couchdb_opts),
    {ok, DbName} = application:get_env(tanuki_backend, database),
    Server = couchbeam:server_connection(Url, Opts),
    {ok, Db} = couchbeam:open_or_create_db(Server, DbName, []),
    install_designs(Db),
    State = #state{server=Server, database=Db},
    {ok, State}.

handle_call({fetch_document, DocId}, _From, #state{database=Db}=State) ->
    case couchbeam:open_doc(Db, DocId) of
        {ok, Doc} -> {reply, {ok, Doc}, State};
        {error, Reason} -> {reply, {error, Reason}, State}
    end;
handle_call(all_tags, _From, #state{database=Db}=State) ->
    Options = [{group_level, 1}],
    {ok, Rows} = couchbeam_view:fetch(Db, {"assets", "tags"}, Options),
    {reply, Rows, State};
handle_call({by_checksum, Checksum}, _From, #state{database=Db}=State) ->
    Options = [{key, list_to_binary(Checksum)}],
    {ok, Rows} = couchbeam_view:fetch(Db, {"assets", "by_checksum"}, Options),
    {reply, Rows, State};
handle_call({by_date, StartDate, EndDate}, _From, #state{database=Db}=State) ->
    Options = [
        {start_key, StartDate},
        {end_key, EndDate}
    ],
    {ok, Rows} = couchbeam_view:fetch(Db, {"assets", "by_date"}, Options),
    {reply, Rows, State};
handle_call({by_tag, Tag}, _From, #state{database=Db}=State) ->
    Options = [{key, list_to_binary(Tag)}],
    {ok, Rows} = couchbeam_view:fetch(Db, {"assets", "by_tag"}, Options),
    {reply, Rows, State};
handle_call({by_tags, Tags}, _From, #state{database=Db}=State) ->
    BinTags = [list_to_binary(Tag) || Tag <- Tags],
    Options = [{keys, BinTags}],
    {ok, Rows} = couchbeam_view:fetch(Db, {"assets", "by_tag"}, Options),
    {reply, Rows, State}.

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

install_designs(Db) ->
    % Look for .json files in our private views directory and insert them
    % directly into CouchDB. They are presumed to be our design documents
    % and thus needed for general operation.
    PrivPath = code:priv_dir(tanuki_backend),
    ViewsDir = filename:join(PrivPath, "views"),
    InsertDocument = fun(Filename) ->
        Filepath = filename:join([ViewsDir, Filename]),
        {ok, Binary} = file:read_file(Filepath),
        Json = couchbeam_ejson:decode(Binary),
        DocId = bitstring_to_list(couchbeam_doc:get_id(Json)),
        case couchbeam:doc_exists(Db, DocId) of
            true -> ok;
            false -> {ok, _Doc1} = couchbeam:save_doc(Db, Json)
        end
    end,
    ConsiderDocument = fun(Filename) ->
        case filename:extension(Filename) of
            ".json" -> InsertDocument(Filename);
            _ -> ok
        end
    end,
    ViewPath = filename:absname(ViewsDir),
    {ok, Filenames} = file:list_dir(ViewPath),
    [ConsiderDocument(Filename) || Filename <- Filenames],
    ok.
