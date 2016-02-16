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
-module(tanuki_backend_app).
-behaviour(application).
-export([start/2, stop/1, ensure_schema/1]).
-include_lib("tanuki_backend/include/records.hrl").

start(_Type, _Args) ->
    Nodes = [node()],
    ensure_schema(Nodes),
    ensure_mnesia(Nodes),
    ok = mnesia:wait_for_tables([thumbnails, thumbnail_dates, thumbnail_counter], 5000),
    nitrogen_sup:start_link(),
    tanuki_backend_sup:start_link().

stop(_) ->
    ok.

% @doc Ensure the schema and our table is installed in mnesia.
ensure_schema(Nodes) ->
    % Create the schema if it does not exist
    case mnesia:system_info(schema_version) of
        {0, 0} ->
            ok = mnesia:create_schema(Nodes);
        {_, _} ->
            ok
    end,
    EnsureTables = fun() ->
        Tables = mnesia:system_info(tables),
        case lists:member(thumbnails, Tables) of
            false ->
                {atomic, ok} = mnesia:create_table(thumbnails, [
                    % first field of the record is the table key
                    {attributes, record_info(fields, thumbnails)}
                    % ram_copies only for now
                    % {disc_copies, Nodes}
                ]),
                {atomic, ok} = mnesia:create_table(thumbnail_dates, [
                    % first field of the record is the table key
                    {attributes, record_info(fields, thumbnail_dates)},
                    {type, ordered_set}
                ]),
                {atomic, ok} = mnesia:create_table(thumbnail_counter, [
                    {attributes, record_info(fields, thumbnail_counter)}
                ]),
                ok;
            true ->
                ok
        end
    end,
    % Create our table if it does not exist
    case mnesia:system_info(is_running) of
        no ->
            rpc:multicall(Nodes, application, start, [mnesia]),
            EnsureTables(),
            rpc:multicall(Nodes, application, stop, [mnesia]);
        _ ->
            EnsureTables()
    end.

% @doc Ensure the mnesia application is running.
ensure_mnesia(Nodes) ->
    case mnesia:system_info(is_running) of
        no ->
            rpc:multicall(Nodes, application, start, [mnesia]);
        _ ->
            ok
    end.
