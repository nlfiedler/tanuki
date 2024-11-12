# Project History

Original idea was inspired by [perkeep](https://perkeep.org) (née camlistore).
However, installing [Go](https://golang.org) on the server system in use at the
time, [Solaris](https://www.oracle.com/solaris/), was too difficult. Given the
operating system there would not be any readily available software to serve the
purpose. Would later learn that this application space is referred to as
"digital asset management."

## July 2014

[Python](https://www.python.org) script to ingest assets, insert records into
[CouchDB](http://couchdb.apache.org).
[Erlang](http://www.erlang.org)/[Nitrogen](http://nitrogenproject.com) interface
to display assets by one tag, in a long, single-column list.

## March 2015

Replaced the Python ingestion script with Erlang, no more Python in the main
application.

## January 2017

Replaced Erlang/Nitrogen code with [Elixir](https://elixir-lang.org) and
[Phoenix](https://phoenixframework.org); support querying by multiple tags;
pagination of assets; basic asset editor.

## March 2017

Replace static Elixir/Phoenix web pages with dynamic [Elm](http://elm-lang.org)
frontend, supporting multiple tags, locations, and years. Hides less frequently
used tags and locations by default, with expanders for showing everything. Form
input validation for asset edit page.

## November 2017

[Node.js](https://nodejs.org/) rewrite of the Elixir/Phoenix backend, using
[PouchDB](https://pouchdb.com) instead of CouchDB; Elm frontend still in place.

In parallel, an [Electron](https://www.electronjs.org)-based desktop application, named [mujina](https://github.com/nlfiedler/mujina), was crafted to use the REST API, then later the GraphQL API. This effort was abandoned in 2020.

## March 2018

Replaced REST-like interface with [GraphQL](https://graphql.org).

Introduced automatic data migration to perform database schema upgrades.

## May 2018

Change asset storage design from sharding by SHA256 checksum to something akin
to Apple Photos (see [DESIGN.md](./DESIGN.md)).

## October 2018

Replace frontend Elm code with [ReasonML](https://reasonml.github.io/en/).

## December 2019

Started to rewrite the Node.js backend in [Rust](https://www.rust-lang.org).

## February 2020

Started rewrite in [Dart](https://dart.dev) and [Flutter](https://flutter.dev)
with the intention of replacing all of the Node.js and ReasonML code. Would
later abandon using Dart (see architecture decision records).

## April 2020

Replaced backend Node.js code with [Rust](https://www.rust-lang.org).

## September 2020

Replaced frontend ReasonML code with [Flutter](https://flutter.dev).

## April 2023

Finally have an easy-to-use method for assigning tags and locations to new assets.

## February 2024

* Location property is now a record with label, city, and region. Populated automatically on import by reverse geocoding the GPS coordinates in the asset, if any.
* Dump and load for simple backup/restore and data migration during schema changes.
* GraphQL-only bulk edit operation to perform various changes across matching assets.

## October 2024

Replaced Flutter front-end with [Leptos](https://leptos.dev), project is now entirely Rust.

Finally added web-based interface for bulk edit of many assets.

## November 2024

Replicated the advanced text-based query support found in [Perkeep](https://perkeep.org).
