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
% Correct orientation of all image assets.
%

-module(correct_orientation).

-export([main/1]).

main([Filename]) ->
    % Consult the Erlang terms file that contains our configuration,
    % which looks something like this:
    %
    % {assets_dir, "/zeniba/shared/tanuki"}.
    % {couchdb_url, "http://192.168.1.1:5984"}.
    % {couchdb_opts, [{basic_auth, {"admin", "secr3t"}}]}.
    %
    case file:consult(Filename) of
        {ok, Terms} ->
            process_all(Terms);
        {error, enoent} ->
            io:format("error: file ~p not found~n", [Filename]),
            halt(1)
        % else, surface the error
    end;
main([]) ->
    io:format("Usage: correct_orientation <config-file>~n"),
    ok.

process_all(Config) ->
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
    {ok, Rows} = couchbeam_view:all(Db, []),
    io:format("~p matching records~n", [length(Rows)]),
    AssetsDir = proplists:get_value(assets_dir, Config),
    lists:foreach(fun(Row) ->
        DocId = couchbeam_doc:get_value(<<"id">>, Row),
        process_one(Db, DocId, AssetsDir)
    end, Rows).

% For the given document, correct the orientation of the image, if
% necessary. Does nothing if the image is already correctly oriented, or if
% the asset is not an image.
process_one(Db, DocId, AssetsDir) ->
    {ok, Doc} = couchbeam:open_doc(Db, DocId),
    case couchbeam_doc:get_value(<<"mimetype">>, Doc) of
        <<"image/jpeg">> ->
            OldChecksumBin = couchbeam_doc:get_value(<<"sha256">>, Doc),
            OldChecksum = binary_to_list(OldChecksumBin),
            AssetPath = convert_to_relative_path(AssetsDir, OldChecksum),
            NewBinary = correct_orientation(AssetPath),
            NewChecksum = compute_checksum(NewBinary),
            update_asset(AssetsDir, Db, Doc, NewBinary, OldChecksum, NewChecksum);
        _ ->
            io:format("skipping non-image ~s~n", [DocId])
    end.

% Load the image into memory and auto-orient it, returning the new data.
% Returns the existing data if orientation is not successful or necessary.
correct_orientation(Filepath) ->
    % Fail fast if unable to read file.
    {ok, Binary0} = file:read_file(Filepath),
    case emagick_rs:requires_orientation(Binary0) of
        false ->
            % Return the original binary, rather than the one that has been
            % handled by ImageMagick, which invariably modifies the data,
            % and thus changes the checksum.
            Binary0;
        true ->
            % Attempt to auto-orient the image, but tolerate assets that
            % are not images, and hence cannot be rotated.
            case emagick_rs:auto_orient(Binary0) of
                {ok, Binary1} -> Binary1;
                {error, _Reason} ->
                    Binary0
            end
    end.

% Generate the path to the asset with the given checksum.
convert_to_relative_path(AssetsDir, Checksum) ->
    Part1 = string:sub_string(Checksum, 1, 2),
    Part2 = string:sub_string(Checksum, 3, 4),
    Part3 = string:sub_string(Checksum, 5),
    filename:join([AssetsDir, Part1, Part2, Part3]).

% Compute the SHA256 for the given binary and return as a hex string.
compute_checksum(Binary) ->
    Digest = crypto:hash(sha256, Binary),
    lists:flatten([io_lib:format("~2.16.0b", [X]) || <<X:8>> <= Digest]).

% If the asset has been modified, save it to the new location and update
% the database entry to reflect the new checksum.
update_asset(_AssetsDir, _Db, Doc, _Binary, _C, _C) ->
    DocId = couchbeam_doc:get_id(Doc),
    io:format("does not require updating: ~s~n", [DocId]);
update_asset(AssetsDir, Db, Doc, Binary, OldChecksum, NewChecksum) ->
    OldAssetPath = convert_to_relative_path(AssetsDir, OldChecksum),
    NewAssetPath = convert_to_relative_path(AssetsDir, NewChecksum),
    NewAssetDir = filename:dirname(NewAssetPath),
    case filelib:is_dir(NewAssetDir) of
        true  -> ok;
        false -> ok = filelib:ensure_dir(NewAssetPath)
    end,
    ok = file:write_file(NewAssetPath, Binary),
    NewDoc = couchbeam_doc:set_value(<<"sha256">>, list_to_binary(NewChecksum), Doc),
    couchbeam:save_doc(Db, NewDoc),
    ok = file:delete(OldAssetPath),
    DocId = couchbeam_doc:get_id(Doc),
    io:format("updated ~s~n", [DocId]).
