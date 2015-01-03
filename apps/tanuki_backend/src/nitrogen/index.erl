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
-module(index).
-compile(export_all).
-include_lib("nitrogen_core/include/wf.hrl").

main() ->
    PrivPath = code:priv_dir(tanuki_backend),
    #template { file=PrivPath ++ "/templates/bare.html" }.

title() -> "Welcome to Tanuki".

body() ->
    #container_12 { body=[
        #grid_8 { alpha=true, prefix=2, suffix=2, omega=true, body=inner_body() }
    ]}.

inner_body() ->
    Rows = tanuki_backend:all_tags(),
    MakeLink = fun(Row) ->
        Label = bitstring_to_list(couchbeam_doc:get_value(<<"key">>, Row)),
        Url = "/tag?tags=" ++ Label,
        [#listitem { body=#link { title=Label, text=Label, url=Url }}]
    end,
    [
        #h1 { text="Welcome to Tanuki" },
        #p{},
        #list{
            numbered=false,
            body=[MakeLink(Row) || Row <- Rows]
        }
    ].
