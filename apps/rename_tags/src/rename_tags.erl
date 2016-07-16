%% -*- erlang -*-
%% -------------------------------------------------------------------
%%
%% Copyright (c) 2016 Nathan Fiedler
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
%
% Rename a tag in all matching documents.
%

-module(rename_tags).

-export([main/1]).

main([Filename, OldTag, NewTag]) ->
    % Consult the Erlang terms file that contains our configuration,
    % which looks something like this:
    %
    % {couchdb_url, "http://192.168.1.1:5984"}.
    % {couchdb_opts, [{basic_auth, {"admin", "secr3t"}}]}.
    %
    case file:consult(Filename) of
        {ok, Terms} ->
            process_all(Terms, OldTag, NewTag);
        {error, enoent} ->
            io:format("File ~p not found", [Filename]),
            halt(1)
        % else, surface the error
    end;
main([]) ->
    io:format("Usage: rename_tags <config-file> <old-tag> <new-tag>~n"),
    ok.

process_all(Config, OldTag, NewTag) ->
    OldBin = list_to_binary(OldTag),
    NewBin = list_to_binary(NewTag),
    ok = application:load(jsx),
    ok = application:load(couchbeam),
    {ok, _Started} = application:ensure_all_started(couchbeam),
    Url = proplists:get_value(couchdb_url, Config),
    Opts = case proplists:get_value(couchdb_opts, Config) of
        undefined -> [];
        Val -> Val
    end,
    Server = couchbeam:server_connection(Url, Opts),
    {ok, Db} = couchbeam:open_or_create_db(Server, "tanuki", []),
    Options = [{key, OldBin}],
    {ok, Rows} = couchbeam_view:fetch(Db, {"assets", "by_tag"}, Options),
    io:format("~p matching records~n", [length(Rows)]),
    lists:foreach(fun(Row) ->
        DocId = couchbeam_doc:get_value(<<"id">>, Row),
        process_one(Db, DocId, OldBin, NewBin)
    end, Rows).

% For the given document, rename the old tag to the new tag (both binary).
process_one(Db, DocId, OldTag, NewTag) ->
    {ok, Doc} = couchbeam:open_doc(Db, DocId),
    Tags = couchbeam_doc:get_value(<<"tags">>, Doc),
    RenamedTags = lists:delete(OldTag, Tags) ++ [NewTag],
    SortedTags = lists:sort(RenamedTags),
    NewDoc = couchbeam_doc:set_value(<<"tags">>, SortedTags, Doc),
    couchbeam:save_doc(Db, NewDoc),
    io:format("updated ~p~n", [DocId]),
    ok.
