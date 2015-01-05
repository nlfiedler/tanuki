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
    % TODO: get these showing side-by-side (see style.css example in nitrogenproject.com source)
        #grid_4 { alpha=true, prefix=1, body=tag_list() },
        #grid_8 { suffix=1, omega=true, body=inner_body() }
    ]}.

tag_list() ->
    % TODO: show selected tags in a sidebar, with 'x' to remove each from the list
    Tag = wf:q(tags),
    Tags = string:tokens(Tag, ","),
    OtherTags = compute_other_tags(Tags),
    MakeTagLink = fun(Row) ->
        Label = bitstring_to_list(couchbeam_doc:get_value(<<"key">>, Row)),
        Url = "/tag?tags=" ++ string:join([Label] ++ Tags, ","),
        [#listitem { body=#link { title=Label, text=Label, url=Url }}]
    end,
    [
        #h3 { text="Tags" },
        #p {},
        #list {
            numbered=false,
            body=[MakeTagLink(Row) || Row <- OtherTags]
        }
    ].

inner_body() ->
    Tag = wf:q(tags),
    Tags = string:tokens(Tag, ","),
    Title = string:join(Tags, ", "),
    MakeAssetLink = fun(Row) ->
        Id = bitstring_to_list(couchbeam_doc:get_value(<<"id">>, Row)),
        Values = couchbeam_doc:get_value(<<"value">>, Row),
        DateString = tanuki_backend:date_list_to_string(hd(Values), date_only),
        FileName = bitstring_to_list(hd(tl(Values))),
        Label = io_lib:format("~s - ~s", [FileName, DateString]),
        Url = "/asset?id=" ++ Id,
        % prepare the thumbnail
        Checksum = bitstring_to_list(hd(tl(tl(Values)))),
        Checkslash = string:substr(Checksum, 1, 2) ++ "/" ++
            string:substr(Checksum, 3, 2) ++ "/" ++ string:substr(Checksum, 5),
        ImageSrc = "/thumbnails/" ++ Checkslash,
        % TODO: would be nice to have these flow horizontally and wrap around
        % TODO: is there a more appropriate element type than container_12?
        [#container_12 {
            class="img",
            body=#link {
                url=Url,
                body=#image {
                    image=ImageSrc,
                    alt=FileName
                }
            }
        }, #container_12 {
            class="desc",
            body=Label
        }]
    end,
    [
        #h1 { text="Assets tagged with " ++ Title },
        #p {},
        [MakeAssetLink(Row) || Row <- tanuki_backend:by_tags(Tags)]
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
