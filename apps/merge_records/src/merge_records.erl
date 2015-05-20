%% -*- erlang -*-
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
%
% Find and merge duplicate records, where duplicate means they have the
% same sha256 value. Preference is given to the earliest imported document,
% and merging consists of joining the set of tags of each of the duplicate
% records. The location of the original record is retained.
%
% Prerequisites:
% * couchbeam -- https://github.com/benoitc/couchbeam
%

-module(merge_records).

-export([main/1]).

main([Filename]) ->
    % Consult the Erlang terms file that contains our configuration,
    % which looks something like this:
    %
    % {couchdb_url, "http://192.168.1.1:5984"}.
    % {couchdb_opts, [{basic_auth, {"admin", "secr3t"}}]}.
    %
    case file:consult(Filename) of
        {ok, Terms} ->
            find_duplicates(Terms);
        {error, enoent} ->
            io:format("File ~p not found", [Filename]),
            halt(1)
        % else, surface the error
    end;
main([]) ->
    io:format("Usage: merge_records <config-file>~n"),
    ok.

find_duplicates(Config) ->
    ok = application:load(jsx),
    ok = application:load(couchbeam),
    ok = couchbeam:start(),
    Url = proplists:get_value(couchdb_url, Config),
    Opts = case proplists:get_value(couchdb_opts, Config) of
        undefined -> [];
        Val -> Val
    end,
    Server = couchbeam:server_connection(Url, Opts),
    {ok, Db} = couchbeam:open_or_create_db(Server, "tanuki", []),
    {ok, QueryResults} = couchbeam_view:fetch(Db, {"assets", "by_checksum"}, []),
    io:format("Number of records: ~p~n", [length(QueryResults)]),
	% extract the checksum and document id from the results
    RawFields = lists:map(fun(Elem) ->
            {couchbeam_doc:get_value(<<"key">>, Elem),
             couchbeam_doc:get_value(<<"id">>, Elem)}
        end, QueryResults),
    % collect those with the same checksum into lists
    ByChecksum = lists:foldl(fun({Key, Id}, AccIn) ->
            case maps:is_key(Key, AccIn) of
                true ->
                    maps:put(Key, maps:get(Key, AccIn) ++ [Id], AccIn);
                false ->
                    maps:put(Key, [Id], AccIn)
            end
        end, #{}, RawFields),
    % filter those that have a list of size one (i.e. not duplicated)
    Duplicates = lists:filter(fun({_Key, IdList}) ->
            length(IdList) > 1
        end, maps:to_list(ByChecksum)),
    io:format("Number of duplicated checksums: ~p~n", [length(Duplicates)]),
    DuplicateCount = lists:foldl(fun({_Key, IdList}, AccIn) ->
            AccIn + length(IdList)
        end, 0, Duplicates),
    io:format("Number of duplicate records: ~p~n", [DuplicateCount]),
    % process each batch of duplicates by checksum
    [process_duplicates(Db, Key, IdList) || {Key, IdList} <- Duplicates],
	ok.

process_duplicates(Db, Key, IdList) ->
    io:format("~p => ~p~n", [Key, length(IdList)]),
    FetchDoc = fun(DocId) ->
        {ok, Doc} = couchbeam:open_doc(Db, DocId),
        Doc
    end,
    % fetch the actual documents
    Documents = [FetchDoc(Id) || Id <- IdList],
    % collect all of the tags
    AllTags = maps:keys(lists:foldl(fun(Doc, AccIn) ->
            Tags = couchbeam_doc:get_value(<<"tags">>, Doc),
            lists:foldl(fun(Tag, AccIn2) ->
                case maps:is_key(Tag, AccIn2) of
                    true -> AccIn2;
                    false -> maps:put(Tag, 1, AccIn2)
                end
            end, AccIn, Tags)
        end, #{}, Documents)),
    io:format("~p: ~p~n", [Key, AllTags]),
    % select the earliest imported document
    DocDates = [{couchbeam_doc:get_value(<<"import_date">>, Doc), Doc} || Doc <- Documents],
    SortedByDate = lists:keysort(1, DocDates),
    {_Date, Earliest} = hd(SortedByDate),
    io:format("Earliest: ~p~n", [couchbeam_doc:get_value(<<"import_date">>, Earliest)]),
    % should probably get first non-empty location and set that in the earliest doc
    % save the merged tags into the earliest document
    NewDoc = couchbeam_doc:set_value(<<"tags">>, AllTags, Earliest),
    couchbeam:save_doc(Db, NewDoc),
    Duplicates = [Doc || {_D, Doc} <- tl(SortedByDate)],
    io:format("To be deleted: ~p~n", [length(Duplicates)]),
    % remove all of the duplicates
    [couchbeam:delete_doc(Db, Doc) || Doc <- Duplicates],
    ok.
