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
-module(tanuki_thumbnail_handler).
-export([init/3, handle/2, terminate/3]).

init({_Transport, http}, Req, _Opts) ->
    {ok, Req, no_state}.

handle(Req, State) ->
    % {path, <<"/thumbnails/4f/40/25b...">>}
    % {path_info, [<<"4f">>, <<"40">>, <<"25b...">>]}
    % extract the checksum from the request URL
    {PathInfo, Req2} = cowboy_req:path_info(Req),
    PathInfoStr = [bitstring_to_list(P) || P <- PathInfo],
    Checksum = string:join(PathInfoStr, ""),
    RelativePath = filename:join(PathInfoStr),
	% retrieve the thumbnail, set as response body
    {ok, Binary, Mimetype} = tanuki_backend:produce_thumbnail(Checksum, RelativePath),
    {ok, Req3} = cowboy_req:reply(200, [
        {<<"content-type">>, Mimetype}
    ], Binary, Req2),
    {ok, Req3, State}.

terminate(_Reason, _Req, _State) ->
    ok.
