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
-module(nitrogen_sup).
-behaviour(supervisor).
-export([start_link/0, init/1]).

%% Helper macro for declaring children of supervisor
-define(CHILD(I, Type), {I, {I, start_link, []}, permanent, 5000, Type, [I]}).

%% ===================================================================
%% API functions
%% ===================================================================

start_link() ->
    supervisor:start_link({local, ?MODULE}, ?MODULE, []).

%% ===================================================================
%% Supervisor callbacks
%% ===================================================================

init([]) ->
    %% Start the Process Registry...
    application:start(crypto),
    application:start(nprocreg),
    application:start(ranch),

    %% Start Cowboy...
    application:start(cowboy),
    {ok, BindAddress} = application:get_env(cowboy, bind_address),
    {ok, Port} = application:get_env(cowboy, port),
    {ok, ServerName} = application:get_env(cowboy, server_name),
    {ok, DocRoot} = application:get_env(cowboy, document_root),
    {ok, StaticPaths} = application:get_env(cowboy, static_paths),

    % the document_root value already has 'priv' so use priv_dir parent
    PrivParent = filename:dirname(code:priv_dir(tanuki_backend)),
    DocRoot2 = filename:join(PrivParent, DocRoot),
    io:format("Starting Cowboy Server (~s) on ~s:~p, root: '~s'~n",
              [ServerName, BindAddress, Port, DocRoot2]),

    Dispatch = init_dispatch(DocRoot2, StaticPaths),
    {ok, _} = cowboy:start_http(http, 100, [{port, Port}], [
        {env, [{dispatch, Dispatch}]},
        {max_keepalive, 50}
    ]),

    {ok, { {one_for_one, 5, 10}, []} }.

init_dispatch(DocRoot, StaticPaths) ->
    Handler = cowboy_static,
    StaticDispatches = lists:map(fun(Dir) ->
        Path = reformat_path(Dir),
        Opts = [
            {mimetypes, {fun mimetypes:path_to_mimes/2, default}}
            | localized_dir_file(DocRoot, Dir)
        ],
        {Path, Handler, Opts}
    end, StaticPaths),

    %% HandlerModule will end up calling HandlerModule:handle(Req,HandlerOpts)
    HandlerModule = nitrogen_cowboy,
    HandlerOpts = [],

    %% Serve up the assets from the configured directory, providing a
    %% function to produce the appropriate mimetype, and a suitable ETag.
    {ok, AssetsDir} = application:get_env(tanuki_backend, assets_dir),
    AssetsEntry = {"/assets/[...]", cowboy_static,
        [{directory, AssetsDir},
         {mimetypes, {fun tanuki_backend:path_to_mimes/2, default}},
         {etag, {fun tanuki_backend:generate_etag/2, strong_etag_extra}}]},
    %% Install our handler for thumbnails
    ThumbnailsEntry = {"/thumbnails/[...]", tanuki_thumbnail_handler, []},

    %% Start Cowboy...
    %% NOTE: According to Loic, there's no way to pass the buck back to cowboy
    %% to handle static dispatch files so we want to make sure that any large
    %% files get caught in general by cowboy and are never passed to the nitrogen
    %% handler at all. In general, your best bet is to include the directory in
    %% the static_paths section of cowboy.config
    Dispatch = [
        %% Nitrogen will handle everything that's not handled in the StaticDispatches
        {'_', StaticDispatches ++ [
            AssetsEntry,
            ThumbnailsEntry,
            {'_', HandlerModule , HandlerOpts}
        ]}
    ],
    cowboy_router:compile(Dispatch).

localized_dir_file(DocRoot, Path) ->
    NewPath = case hd(Path) of
        $/ -> DocRoot ++ Path;
        _ -> DocRoot ++ "/" ++ Path
    end,
    _NewPath2 = case lists:last(Path) of
        $/ -> [{directory, NewPath}];
        _ ->
            Dir = filename:dirname(NewPath),
            File = filename:basename(NewPath),
            [
                {directory, Dir},
                {file, File}
            ]
    end.

%% Ensure the paths start with /, and if a path ends with /, then add "[...]" to it
reformat_path(Path) ->
    Path2 = case hd(Path) of
        $/ -> Path;
        $\ -> Path;
        _ -> [$/|Path]
    end,
    Path3 = case lists:last(Path) of
        $/ -> Path2 ++ "[...]";
        $\ -> Path2 ++ "[...]";
        _ -> Path2
    end,
    Path3.
