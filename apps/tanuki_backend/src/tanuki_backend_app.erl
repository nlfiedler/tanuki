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
-export([start/2, stop/1]).
-include("records.hrl").

start(_Type, _Args) ->
    % Nodes = [node()|nodes()],
    % ensure_schema(Nodes),
    % ensure_mnesia(Nodes),
    nitrogen_sup:start_link(),
    tanuki_backend_sup:start_link().

stop(_) ->
    ok.

% % @doc Ensure the schema is installed in mnesia.
% ensure_schema(Nodes) ->
%     case mnesia:system_info(schema_version) of
%         {0, 0} ->
%             ok = mnesia:create_schema(Nodes),
%             rpc:multicall(Nodes, application, start, [mnesia]),
%             mnesia:create_table(thumbnails, [
%                 {attributes, record_info(fields, thumbnails)},
%                 {index, [#thumbnails.sha256]},
%                 {disc_copies, Nodes}]),
%             rpc:multicall(Nodes, application, stop, [mnesia]);
%         {_, _} ->
%             ok
%     end.

% % @doc Ensure the mnesia application is running.
% ensure_mnesia(Nodes) ->
%     case mnesia:system_info(is_running) of
%         no ->
%             rpc:multicall(Nodes, application, start, [mnesia]);
%         _ ->
%             ok
%     end.
