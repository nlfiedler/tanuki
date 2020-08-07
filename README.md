# Tanuki

A system for importing, storing, categorizing, browsing, displaying, and
searching files, primarily images and videos. Attributes regarding the files are
stored in a key-value store. Designed to store millions of files. Provides a
simple web interface with basic browsing and editing capabilities.

## Building and Testing

### Prerequisites

* [Rust](https://www.rust-lang.org) stable (2018 edition)
* [Node.js](https://nodejs.org/) LTS
* [Gulp](https://gulpjs.com) CLI: `npm -g install gulp-cli`
* [Flutter](https://flutter.dev) beta channel
    - Enable the **web** configuration

#### Example for macOS

This example assumes you are using [Homebrew](http://brew.sh) to install the
dependencies, which provides up-to-date versions of everything needed. The
`xcode-select --install` is there just because the command-line tools sometimes
get out of date, and some of the dependencies will fail to build without them.

```shell
$ xcode-select --install
$ brew install node
$ npm -g install gulp-cli
```

### Building, Testing, Starting the Backend

```shell
$ cargo update
$ cargo build
$ cargo test
$ RUST_LOG=info cargo run --release
```

For more verbose debugging output, use `RUST_LOG=debug` in the command above.
For extremely verbose logging, use `RUST_LOG=trace` which will dump large
volumes of output.

### Building, Testing, Starting the Frontend

#### Flutter

```shell
$ flutter pub get
$ flutter pub run environment_config:generate
$ flutter test
$ flutter run -d chrome
```

#### ReasonML

```shell
$ npm install
$ gulp build
```

### Updating the GraphQL PPX schema

The ReasonML support for GraphQL uses a JSON formatted representation of the
schema, which is generated using the following command (after starting a local
server in another window):

```shell
$ npx apollo-codegen introspect-schema http://localhost:3000/graphql --output graphql_schema.json
```

### environment_config

The frontend has some configuration that is set up at build time using the
[environment_config](https://pub.dev/packages/environment_config) package. The
generated file (`lib/environment_config.dart`) is not version controlled, and
the values can be set at build-time using either command-line arguments or
environment variables. See the `pubspec.yaml` for the names and the
`environment_config` README for instructions.

## Deploying

### Using Docker

The base directory contains a `docker-compose.yml` file which is used to build
the application in stages and produce a relatively small final image.

On the build host:

```shell
$ docker-compose build --pull --build-arg BASE_URL=http://192.168.1.1:3000
$ docker image rm 192.168.1.1:5000/tanuki_app
$ docker image tag tanuki_app 192.168.1.1:5000/tanuki_app
$ docker push 192.168.1.1:5000/tanuki_app
```

On the server, with a production version of the `docker-compose.yml` file:

```shell
$ docker-compose pull
$ docker-compose rm -f -s
$ docker-compose up -d
```

## Tools

### Finding Outdated Crates

Use https://github.com/kbknapp/cargo-outdated and run `cargo outdated`

### License checking

Use the https://github.com/Nemo157/cargo-lichking `cargo` utility. To install:

```shell
$ OPENSSL_ROOT_DIR=`brew --prefix openssl` \
  OPENSSL_LIB_DIR=`brew --prefix openssl`/lib \
  OPENSSL_INCLUDE_DIR=`brew --prefix openssl`/include \
  cargo install cargo-lichking
```

To get the list of licenses, and check for incompatibility:

```shell
$ cargo lichking list
$ cargo lichking check
```

However, need to look for "gpl" manually in the `list` output, as most licenses
that are compatible with MIT/Apache are also compatible with GPL.

## Architecture

Assets stored as-is in date/time formatted directory structure, metadata stored
in a key-value store, an HTTP server backend, and a single-page application for
a front-end. Backend written in Rust, front-end written in
[ReasonML](https://reasonml.github.io/en/), database is
[RocksDB](https://rocksdb.org). The client/server protocol consists mostly of
[GraphQL](https://graphql.org) queries and HTTP requests for asset data.

### Clean Architecture

The backend is designed using the [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html) in which the application is divided into three layers: domain, data, and presentation. The domain layer defines the "policy" or business logic of the application, consisting of entities, use cases, and repositories. The data layer is the interface to the underlying system, defining the data models that are ultimately stored in a database. The presentation layer is what the user generally sees, the web interface, and to some extent, the GraphQL interface.

## Design

### Storage

When an asset is added to the system, several steps are performed:

1. A SHA256 checksum of the file contents is computed.
1. An identifier based on the import date/time, filename extension,
   and [ULID](https://github.com/ulid/spec) is generated.
1. A minimal document, with checksum and identifier, is stored in the database.
1. The asset is stored in a directory structure according to import date/time.
    - e.g. `2017/05/13/1630/01ce0d526z6cyzgm02ap0jv281.jpg`
    - the minutes are rounded to `:00`, `:15`, `:30`, or `:45`
    - the base filename is the ULID
    - the original file extension is retained

#### Benefits

* Assets are stored in directory structure reflecting time and order of addition
    - ULID sorts by time, so order of insertion is retained
* Number of directories and files at any particular level is reasonable
    - at most 96 directories per calendar day
    - files per directory limited to what can be uploaded in 15 minutes
* Can rebuild some of the metadata from the directory structure and file names
    - import date/time from file path
    - media type from extension
    - original date/time from file metadata
* Encoded path as asset ID allows serving asset without database lookup
    - base64 encoded asset path happens to be same length as SHA256
* Avoids filename collisions
    - names like `IMG_1234.JPG` easily collide with other devices

#### Drawbacks

The files are renamed, which might be a bother to some people. In many cases,
the file names are largely irrelevant, as most are of the form `IMG_1234.JPG`.
In other cases, the names are something ridiculous, like
`20150419171116-63EK7JXWKEVMDJVV-P1510081.jpg`, which encodes a date/time and
some seemingly random sequence of letters and numbers. The good news is the
original file name is recorded in the database.

#### Some History

From the very beginning of the project, assets were stored in a directory
structure reflecting the checksum, reminiscent of Git. For instance, if the file
checksum was `938f831fb02b313e7317c1e0631b86108a9e4a197e33d581fb68be91a3c6ce2f`,
then the file would be stored in a directory path `93/8f` with a filename of
`831fb02b313e7317c1e0631b86108a9e4a197e33d581fb68be91a3c6ce2f`. Using the
checksum as the asset identifier made it very easy to serve the asset without a
database lookup.

However, this design had several problems:

* Discarded most information about the asset:
    - file name and extension
    - media type cannot be guessed
    - import date/time
* With only 256 by 256 directories, the files-per-directory scales linearly
    - for 100,000 assets, ~1.5 files in each directory
    - for 1,000,000 assets, ~15 files in each directory
    - for 1,000,000,000 assets, ~15,000 files in each directory
* Looks scary to normal people

## Project History

Original idea was inspired by [perkeep](https://perkeep.org) (née camlistore).
However, installing [Go](https://golang.org) on
[Solaris](https://www.oracle.com/solaris/) was difficult. Given the operating
system there would not be any readily available software to serve the purpose.
Would later learn that this application space is referred to as "digital asset
management."

### July 2014

[Python](https://www.python.org) script to ingest assets, insert records into
[CouchDB](http://couchdb.apache.org).
[Erlang](http://www.erlang.org)/[Nitrogen](http://nitrogenproject.com) interface
to display assets by one tag, in a long, single-column list (i.e. no
pagination).

### March 2015

Replaced the Python ingest script with Erlang, no more Python in the main
application.

### January 2017

Replaced Erlang/Nitrogen code with [Elixir](https://elixir-lang.org) and
[Phoenix](https://phoenixframework.org); support querying by multiple tags;
pagination of assets; basic asset editor.

### March 2017

Replace static Elixir/Phoenix web pages with dynamic [Elm](http://elm-lang.org)
front-end, supporting multiple tags, locations, and years. Hides less frequently
used tags and locations by default, with expanders for showing everything. Form
input validation for asset edit page.

### November 2017

[Node.js](https://nodejs.org/) rewrite of the Elixir/Phoenix backend, using
[PouchDB](https://pouchdb.com) instead of CouchDB; Elm front-end still in place.

### March 2018

Replaced REST-like interface with [GraphQL](https://graphql.org).

Introduced automatic data migration to perform database schema upgrades.

### May 2018

Change asset storage design from sharding by SHA256 checksum to something akin
to Apple Photos (see above).

### October 2018

Replace front-end Elm code with [ReasonML](https://reasonml.github.io/en/).

### December 2019

Started to rewrite the Node.js backend in [Rust](https://www.rust-lang.org).

### February 2020

Started rewrite in [Dart](https://dart.dev) and [Flutter](https://flutter.dev)
with the intention of replacing all of the Node.js and ReasonML code.

### April 2020

Switch from Dart back to Rust. Long live Rust.

### August 2020

Started frontend rewrite in [Flutter](https://flutter.dev).
