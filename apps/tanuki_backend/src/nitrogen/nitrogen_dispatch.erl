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
-module(nitrogen_dispatch).

-export([init_dispatch/0]).

init_dispatch() ->
    {ok, OldDocRoot} = application:get_env(simple_bridge, document_root),
    {ok, StaticPaths} = application:get_env(simple_bridge, static_paths),

    % the document_root value already has 'priv' so use priv_dir parent
    PrivParent = filename:dirname(code:priv_dir(tanuki_backend)),
    DocRoot = filename:join(PrivParent, OldDocRoot),

    StaticDispatches = lists:map(fun(Dir) ->
        Path = reformat_path(Dir),
        Opts = [
            {mimetypes, {fun mimetypes:path_to_mimes/2, default}}
            | localized_dir_file(DocRoot, Dir)
        ],
        {Path, cowboy_static, Opts}
    end, StaticPaths),

    %% Serve up the assets from the configured directory, providing a
    %% function to produce the appropriate mimetype, and a suitable ETag.
    {ok, AssetsDir} = application:get_env(tanuki_backend, assets_dir),
    AssetsEntry = {"/assets/[...]", cowboy_static,
        [{directory, AssetsDir},
         {mimetypes, {fun tanuki_backend:path_to_mimes/2, default}},
         {etag, {fun tanuki_backend:generate_etag/2, strong_etag_extra}}]},
    %% Install our handler for thumbnails
    ThumbnailsEntry = {"/thumbnails/[...]", tanuki_thumbnail_handler, []},

    [
        %% Nitrogen will handle everything that's not handled in the StaticDispatches
        {'_', StaticDispatches ++ [
            AssetsEntry,
            ThumbnailsEntry,
            {'_', nitrogen_cowboy , []}
        ]}
    ].

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
