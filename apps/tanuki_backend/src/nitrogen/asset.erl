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
-module(asset).
-compile(export_all).
-include_lib("nitrogen_core/include/wf.hrl").

main() ->
    PrivPath = code:priv_dir(tanuki_backend),
    #template { file=PrivPath ++ "/templates/bare.html" }.

title() ->
    Id = wf:q(id),
    "Asset " ++ Id.

body() ->
    #container_12 { body=[
        #grid_8 { alpha=true, prefix=2, suffix=2, omega=true, body=inner_body() }
    ]}.

inner_body() ->
    Id = wf:q(id),
    {document, Document} = tanuki_backend:fetch_document(Id),
    Checksum = bitstring_to_list(couchbeam_doc:get_value(<<"sha256">>, Document)),
    Checkslash = string:substr(Checksum, 1, 2) ++ "/" ++
        string:substr(Checksum, 3, 2) ++ "/" ++ string:substr(Checksum, 5),
    ImageSrc = "/thumbnails/" ++ Checkslash,
    Label = bitstring_to_list(couchbeam_doc:get_value(<<"file_name">>, Document)),
    ImageUrl = "/assets/" ++ Checkslash,
    [
        #h1 { text="Asset " ++ Id },
        #p{},
        #link { title=Label, text=Label, url=ImageUrl, body=#image{ image=ImageSrc } }
    ].
