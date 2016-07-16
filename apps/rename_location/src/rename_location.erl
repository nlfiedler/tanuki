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
% Rename a location in all matching documents.
%

-module(rename_location).

-export([main/1]).

main([Filename, OldLoc, NewLoc]) ->
    % Consult the Erlang terms file that contains our configuration,
    % which looks something like this:
    %
    % {couchdb_url, "http://192.168.1.1:5984"}.
    % {couchdb_opts, [{basic_auth, {"admin", "secr3t"}}]}.
    %
    case file:consult(Filename) of
        {ok, Terms} ->
            process_all(Terms, OldLoc, NewLoc);
        {error, enoent} ->
            io:format("File ~p not found", [Filename]),
            halt(1)
        % else, surface the error
    end;
main([]) ->
    io:format("Usage: rename_location <config-file> <old> <new>~n"),
    ok.

process_all(Config, OldLoc, NewLoc) ->
    OldBin = list_to_binary(OldLoc),
    NewBin = list_to_binary(NewLoc),
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
    % would have used such a view, if it existed at the time
    % Options = [{key, OldBin}],
    % {ok, Rows} = couchbeam_view:fetch(Db, {"assets", "by_location"}, Options),
    {ok, Rows} = couchbeam_view:all(Db, []),
    io:format("~p matching records~n", [length(Rows)]),
    lists:foreach(fun(Row) ->
        DocId = couchbeam_doc:get_value(<<"id">>, Row),
        process_one(Db, DocId, OldBin, NewBin)
    end, Rows).

% For the given document, rename the location (both old/new are binary).
process_one(Db, DocId, OldLoc, NewLoc) ->
    {ok, Doc} = couchbeam:open_doc(Db, DocId),
    case couchbeam_doc:get_value(<<"location">>, Doc) of
        L when L == OldLoc ->
            NewDoc = couchbeam_doc:set_value(<<"location">>, NewLoc, Doc),
            couchbeam:save_doc(Db, NewDoc),
            io:format("updated ~p~n", [DocId]);
        _ -> ok
    end.
