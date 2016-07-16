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
% Remove a tag and set as topic in all matching documents.
%

-module(tag_to_topic).

-export([main/1]).

main([Filename, Tag]) ->
    % Consult the Erlang terms file that contains our configuration,
    % which looks something like this:
    %
    % {couchdb_url, "http://192.168.1.1:5984"}.
    % {couchdb_opts, [{basic_auth, {"admin", "secr3t"}}]}.
    %
    case file:consult(Filename) of
        {ok, Terms} ->
            process_all(Terms, Tag);
        {error, enoent} ->
            io:format("error: file ~p not found", [Filename]),
            halt(1)
        % else, surface the error
    end;
main([]) ->
    io:format("Usage: tag_to_topic <config-file> <tag>~n"),
    ok.

process_all(Config, Tag) ->
    BinTag = list_to_binary(Tag),
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
    Options = [{key, BinTag}],
    {ok, Rows} = couchbeam_view:fetch(Db, {"assets", "by_tag"}, Options),
    io:format("~p matching records~n", [length(Rows)]),
    lists:foreach(fun(Row) ->
        DocId = couchbeam_doc:get_value(<<"id">>, Row),
        process_one(Db, DocId, BinTag)
    end, Rows).

% For the given document, remove the tag (in binary) and set as topic.
process_one(Db, DocId, Tag) ->
    {ok, Doc} = couchbeam:open_doc(Db, DocId),
    % If the document has a topic already, do not make any changes.
    case couchbeam_doc:get_value(<<"topic">>, Doc) of
        L when L == undefined orelse L == null ->
            Tags = couchbeam_doc:get_value(<<"tags">>, Doc),
            NewTags = lists:delete(Tag, Tags),
            NewDoc1 = couchbeam_doc:set_value(<<"tags">>, NewTags, Doc),
            NewDoc2 = couchbeam_doc:set_value(<<"topic">>, Tag, NewDoc1),
            couchbeam:save_doc(Db, NewDoc2),
            io:format("updated ~p~n", [DocId]);
        _ -> ok
    end.
