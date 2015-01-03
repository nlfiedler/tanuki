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
-module(tag).
-compile(export_all).
-include_lib("nitrogen_core/include/wf.hrl").

main() ->
    PrivPath = code:priv_dir(tanuki_backend),
    #template { file=PrivPath ++ "/templates/bare.html" }.

title() ->
    Tag = wf:q(tags),
    "Assets tagged with " ++ Tag.

body() ->
    #container_12 { body=[
        % TODO: put the tag links in a sidebar
        % #grid_4 { alpha=true, prefix=2, suffix=2, omega=true, body=tag_list() },
        #grid_8 { alpha=true, prefix=2, suffix=2, omega=true, body=inner_body() }
    ]}.

tag_list() ->
    % TODO: move tags list code here
    % TODO: show selected tags in a sidebar, with 'x' to remove each from the list
    [
    ].

inner_body() ->
    Tag = wf:q(tags),
    Tags = string:tokens(Tag, ","),
    Title = string:join(Tags, ", "),
    OtherTags = compute_other_tags(Tags),
    MakeTagLink = fun(Row) ->
        Label = bitstring_to_list(couchbeam_doc:get_value(<<"key">>, Row)),
        Url = "/tag?tags=" ++ string:join([Label] ++ Tags, ","),
        [#listitem { body=#link { title=Label, text=Label, url=Url }}]
    end,
    MakeAssetLink = fun(Row) ->
        Id = bitstring_to_list(couchbeam_doc:get_value(<<"id">>, Row)),
        Values = couchbeam_doc:get_value(<<"value">>, Row),
        DateString = tanuki_backend:date_list_to_string(hd(Values)),
        FileName = bitstring_to_list(hd(tl(Values))),
        Label = io_lib:format("~p - ~p", [FileName, DateString]),
        Url = "/asset?id=" ++ Id,
        [#listitem { body=#link { title=Label, text=Label, url=Url }}]
    end,
    [
        #h1 { text="Assets tagged with " ++ Title },
        #p{},
        #list{
            numbered=false,
            body=[MakeAssetLink(Row) || Row <- tanuki_backend:by_tags(Tags, unique)]
        },
        #h3 { text="Tags" },
        #p{},
        #list{
            numbered=false,
            body=[MakeTagLink(Row) || Row <- OtherTags]
        }
    ].

%
% @doc Given the list of selected tag names, return all other tags (as a CouchDB
%      row, as from all_tags/0) that are not in the selected list.
%
compute_other_tags(CurrentTags) ->
    AllTags = tanuki_backend:all_tags(),
    Pred = fun(Row) ->
        Elem = bitstring_to_list(couchbeam_doc:get_value(<<"key">>, Row)),
        case lists:member(Elem, CurrentTags) of
            true -> false;
            false -> true
        end
    end,
    lists:filter(Pred, AllTags).
