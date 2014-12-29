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

%% Include the automatically generated plugins directory
-include("plugins.hrl").

%
% Mnesia record for storing thumbnails, and the records for tracking how old a
% particular thumbnail is, for count limiting.
%
-record(thumbnails, {sha256, binary}).
-record(thumbnail_dates, {timestamp, sha256}).
-record(thumbnail_counter, {id = 0, ver = 1}).

%% Include any application-specific custom elements, actions, or validators below
