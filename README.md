# Tanuki

A system for importing, storing, categorizing, browsing, displaying, and
searching assets, primarily images and videos. Attributes regarding the assets
are stored in a key-value store. Provides a simple web interface with basic
browsing and editing capabilities.

## Building and Testing

### Prerequisites

* [Rust](https://www.rust-lang.org) stable (2021 edition)
* [Flutter](https://flutter.dev) **stable** channel

### Initial Setup

Use [fvm](https://pub.dev/packages/fvm) to select a specific version of Flutter
to be installed and used by the application. This is the most reliable method
and produces consistent results when building the application.

```shell
$ brew install dart
$ pub global activate fvm
$ fvm install stable
$ fvm flutter config --enable-macos-desktop
$ fvm flutter config --enable-web
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

Until all of the dependencies have null-safety, must use `--no-sound-null-safety`
option when running the application to avoid an exception.

```shell
$ fvm flutter pub get
$ fvm flutter pub run environment_config:generate
$ fvm flutter test
$ fvm flutter run --no-sound-null-safety -d chrome
```

#### macOS

Building the macOS desktop application on Apple Silicon (M1) requires a
temporary work-around to an issue with installing the Ruby `ffi` library.

```shell
$ arch -x86_64 sudo gem install ffi
```

Until all of the dependencies have null-safety, must use
`--no-sound-null-safety` option when building the application.

```shell
$ fvm flutter build macos --no-sound-null-safety
$ fvm flutter run --no-sound-null-safety -d macos
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
$ docker compose build --pull --build-arg BASE_URL=http://192.168.1.1:3000
$ docker image rm 192.168.1.1:5000/tanuki
$ docker image tag tanuki_app 192.168.1.1:5000/tanuki
$ docker push 192.168.1.1:5000/tanuki
```

On the server, with a production version of the `docker-compose.yml` file:

```shell
$ docker compose down
$ docker compose up --build -d
```

## Tools

### Finding Outdated Crates

Use https://github.com/kbknapp/cargo-outdated and run `cargo outdated -R`

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
a frontend.

* Backend written in [Rust](https://www.rust-lang.org)
* Front-end written in [Flutter](https://flutter.dev)
* Data store is [RocksDB](https://rocksdb.org)
* Wire protocol is [GraphQL](https://graphql.org) and HTTP

### Clean Architecture

The application is designed using the [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html) in which the application is divided into three layers: domain, data, and presentation. The domain layer defines the "policy" or business logic of the application, consisting of entities, use cases, and repositories. The data layer is the interface to the underlying system, defining the data models that are ultimately stored in a database. The presentation layer is what the user generally sees, the web interface, and to some extent, the GraphQL interface.

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

In previous versions of the application, assets were stored in a directory
structure reflecting the checksum of the asset, similar to the object store in
Git. There were two directory levels consisting of two pairs of leading digits
from the checksum, and the filename was the remainder of the checksum.

#### Benefits

* Assets are stored in directory structure reflecting time and order of addition
    - ULID sorts by time, so order of insertion is retained
* Number of directories and files at any particular level is reasonable
    - at most 96 directories per calendar day
    - files per directory limited to what can be processed within 15 minutes
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

## Project History

Original idea was inspired by [perkeep](https://perkeep.org) (née camlistore).
However, installing [Go](https://golang.org) on the server system in use at the
time, [Solaris](https://www.oracle.com/solaris/), was too difficult. Given the
operating system there would not be any readily available software to serve the
purpose. Would later learn that this application space is referred to as
"digital asset management."

### July 2014

[Python](https://www.python.org) script to ingest assets, insert records into
[CouchDB](http://couchdb.apache.org).
[Erlang](http://www.erlang.org)/[Nitrogen](http://nitrogenproject.com) interface
to display assets by one tag, in a long, single-column list.

### March 2015

Replaced the Python ingestion script with Erlang, no more Python in the main
application.

### January 2017

Replaced Erlang/Nitrogen code with [Elixir](https://elixir-lang.org) and
[Phoenix](https://phoenixframework.org); support querying by multiple tags;
pagination of assets; basic asset editor.

### March 2017

Replace static Elixir/Phoenix web pages with dynamic [Elm](http://elm-lang.org)
frontend, supporting multiple tags, locations, and years. Hides less frequently
used tags and locations by default, with expanders for showing everything. Form
input validation for asset edit page.

### November 2017

[Node.js](https://nodejs.org/) rewrite of the Elixir/Phoenix backend, using
[PouchDB](https://pouchdb.com) instead of CouchDB; Elm frontend still in place.

### March 2018

Replaced REST-like interface with [GraphQL](https://graphql.org).

Introduced automatic data migration to perform database schema upgrades.

### May 2018

Change asset storage design from sharding by SHA256 checksum to something akin
to Apple Photos (see above).

### October 2018

Replace frontend Elm code with [ReasonML](https://reasonml.github.io/en/).

### December 2019

Started to rewrite the Node.js backend in [Rust](https://www.rust-lang.org).

### February 2020

Started rewrite in [Dart](https://dart.dev) and [Flutter](https://flutter.dev)
with the intention of replacing all of the Node.js and ReasonML code. Would
later abandon using Dart (see architecture decision records).

### April 2020

Replaced backend Node.js code with [Rust](https://www.rust-lang.org).

### September 2020

Replaced frontend ReasonML code with [Flutter](https://flutter.dev).
