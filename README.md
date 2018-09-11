# Tanuki

A system for importing, storing, categorizing, browsing, displaying, and searching files, primarily images and videos. Attributes regarding the files are stored in a schema-less, document-oriented database. Designed to store millions of files. Provides a simple web interface with basic browsing and editing capabilities.

## Building and Testing

### Prerequisites

* [Node.js](https://nodejs.org/) 8.x
* [Elm](http://elm-lang.org) 0.18

#### Example for MacOS

This example assumes you are using [Homebrew](http://brew.sh) to install the dependencies, which provides up-to-date versions of everything needed. The `xcode-select --install` is there just because the command-line tools sometimes get out of date, and some of the dependencies will fail to build without them.

```shell
$ xcode-select --install
$ brew install node
$ brew install elm
$ npm install -g gulp-cli
```

### Commands

To start an instance configured for development, run the following command.

```shell
$ npm install
$ npm test
$ gulp
```

## Architecture

Assets stored as-is in date/time formatted directory structure, metadata stored in a document-oriented database, an HTTP server backend, and a single-page application for a front-end. Backend written in JavaScript running on [Node.js](https://nodejs.org/), front-end written in Elm, database is [PouchDB](https://pouchdb.com).

## Design

### Storage

When an asset is added to the system, several steps are performed:

1. A SHA256 checksum of the file contents is computed.
1. An identifier based on the import date/time, filename extension, and [ULID](https://github.com/ulid/javascript) is generated.
1. A minimal document, with checksum and identifier, is stored in the database.
1. The asset is stored in a directory structure according to import date/time.
    - e.g. `2017/05/13/1630/01ce0d526z6cyzgm02ap0jv281.jpg`
    - the minutes are rounded to :00, :15, :30, or :45
    - the base filename is the ULID
    - the original file extension is retained

#### Benefits

* Assets are stored in directory structure reflecting time and order of addition
    - ULID sorts by time, so order of insertion is retained
* Number of directories and files at any particular level is reasonable
    - at most 96 directories per calendar day
    - unlikely to have many files per 15 minute block
* Can rebuild some of the metadata from the directory structure and file names
    - import date/time from file path
    - media type from extension
    - original date/time from file metadata
* Encoded path as asset ID allows serving asset without database lookup
    - base64 encoded asset path happens to be same length as SHA256
* Avoids filename collisions
    - names like `IMG_1234.JPG` easily collide with other devices

#### Drawbacks

The files are renamed, which might be a bother to some people. In many cases, the file names are largely irrelevant, as most are of the form `IMG_1234.JPG`. In other cases, the names are something ridiculous, like `20150419171116-63EK7JXWKEVMDJVV-P1510081.jpg`, which encodes a date/time and some seemingly random sequence of letters and numbers. The good news is the original file name is recorded in the database.

#### Some History

From the very beginning of the project, assets were stored in a directory structure reflecting the checksum, reminiscent of Git. For instance, if the file checksum was `938f831fb02b313e7317c1e0631b86108a9e4a197e33d581fb68be91a3c6ce2f`, then the file would be stored in a directory path `93/8f` with a filename of `831fb02b313e7317c1e0631b86108a9e4a197e33d581fb68be91a3c6ce2f`. Using the checksum as the asset identifier made it very easy to serve the asset without a database lookup.

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

Original idea was inspired by [perkeep](https://perkeep.org) (née camlistore). Since installing [Go](https://golang.org) on [Solaris](https://www.oracle.com/solaris/) was difficult, something new was needed. That is, given the operating system there would not be any readily available software to serve the purpose. Would later learn that this application space is referred to as "digital asset management."

### July 2014

[Python](https://www.python.org) script to ingest assets, insert records into [CouchDB](http://couchdb.apache.org). [Erlang](http://www.erlang.org)/[Nitrogen](http://nitrogenproject.com) interface to display assets by one tag, in a long, single-column list (i.e. no pagination).

### March 2015

Replaced the Python ingest script with Erlang, no more Python in the main application.

### January 2017

Replaced Erlang/Nitrogen code with [Elixir](https://elixir-lang.org) and [Phoenix](https://phoenixframework.org); support querying by multiple tags; pagination of assets; basic asset editor.

### March 2017

Replace static Elixir/Phoenix web pages with dynamic [Elm](http://elm-lang.org) front-end, supporting multiple tags, locations, and years. Hides less frequently used tags and locations by default, with expanders for showing everything. Form input validation for asset edit page.

### November 2017

[Node.js](https://nodejs.org/) rewrite of the Elixir/Phoenix backend, using [PouchDB](https://pouchdb.com) instead of CouchDB; Elm front-end still in place.

### March 2018

Replaced REST-like interface with [GraphQL](https://graphql.org).

Introduced automatic data migration to perform database schema upgrades.

### May 2018

Change asset storage design from sharding by SHA256 checksum to something akin to Apple Photos (see above).
