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
-module(nitrogen_main_handler).
-export([run/0, ws_init/0]).

handlers() ->
    %% Put any custom handlers here
    %% See http://nitrogenproject.com/doc/handlers.html
    %% Example:
    %%
    %%   nitrogen:handler(MySecurityHandler, HandlerConfig),
    %%
    %%
    %% The following enables the debug_crash_handler for development. If you
    %% wish to use the default_crash_handler, which just spits out "Internal
    %% Server Error", comment or delete this next line.
    nitrogen:handler(debug_crash_handler, []).

ws_init() ->
    handlers(),
    ok.

run() ->
    handlers(),
    wf_core:run().
