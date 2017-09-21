# Tanuki

A system for importing, storing, categorizing, browsing, displaying, and searching files, primarily images and videos. Attributes regarding the files are stored in a schema-less, document-oriented database. Designed to store millions of files. Primary interface is a web front-end with a simple design suitable for most web browsers.

## Building and Testing

### Prerequisites

* [Elixir](http://elixir-lang.org) 1.3 or higher
* [CouchDB](http://couchdb.apache.org) 1.6.1 or higher
* [ImageMagick](http://www.imagemagick.org) 7.0 or higher
* [FFmpeg](https://ffmpeg.org) 3.2 or higher
* [Rust](https://www.rust-lang.org) 1.18 or higher
* [Node](https://nodejs.org/) 7.x or higher

#### Example for MacOS

This example assumes you are using [Homebrew](http://brew.sh) to install the dependencies, which provides up-to-date versions of everything needed. The `xcode-select --install` is there just because the command-line tools sometimes get out of date, and some of the dependencies will fail to build without them.

```shell
$ xcode-select --install
$ brew install elixir
$ brew install couchdb
$ brew install imagemagick
$ brew install ffmpeg
$ brew install rust
$ brew install node
```

### Configuration

Once the above prerequisites are installed, you will then need to configure the various applications (`tanuki_backend` and `tanuki_incoming`) to override certain settings. To do this, write a `dev.exs` that sets the `assets_dir` and `incoming_dir` as appropriate for your system. See the examples below for some guidance.

```
$ cat apps/tanuki_backend/config/dev.exs
use Mix.Config

config :tanuki_backend,
  assets_dir: '/home/samiam/temp/tanuki'

$ cat apps/tanuki_incoming/config/dev.exs
use Mix.Config

config :tanuki_incoming,
  incoming_dir: '/home/samiam/temp/incoming'
```

### Commands

Once the configuration is in place, you can build everything and run the tests like so:

```shell
$ mix deps.get
$ mix compile
$ mix test
```

To start an instance configured for development, run the following command.

```shell
$ mix phoenix.server
```

The web server will be listening on port `4000`. Be sure to have a CouchDB instance running.

## Architecture

* Set of Erlang/OTP applications written in Elixir.
    - `TanukiIncoming` incorporates digital assets into the system.
        + Reads from an `incoming_dir` any file that is within a directory.
        + The format of the directories is `tagA_tagB_tagC...@some_location`.
        + The asset checksum (SHA256) is used to shard it into storage.
        + Storage location is defined by `assets_dir`.
        + The tags, location, checksum, and other details are stored in CouchDB.
    - `TanukiBackend` provides the interface to the CouchDB documents and views.
        + Handles most of the common database and asset operations.
        + Defines a set of views to query the assets.
        + Caches query results to improve pagination in the web interface.
    - `TanukiWeb` is the Phoenix web interface of the system.
        + [Cowboy](https://github.com/ninenines/cowboy) is the web server.
        + [Plug](https://hexdocs.pm/plug/readme.html) is the backbone.
        + [Phoenix](http://www.phoenixframework.org) is the web framework.
        + [Elm](http://elm-lang.org) is the web front-end language and runtime.
* CouchDB
    - Where the documents describing the assets live.
    - Builds the search-able indexes used to query the documents.
    - [couchbeam](https://github.com/benoitc/couchbeam) is the Erlang client library.

### Why Elixir

Cross-platform, scalable, fault-tolerant, distributed. The language makes coding mistakes less likely through single-assignment variables, immutable data structures, no shared mutable data (granted, mnesia violates this in a sense), and simple control structures. The "let it crash" philosophy encourages a style of programming that leads to more robust software.

This applies to [Erlang](http://www.erlang.org) as well, of course, which is the underpinning of Elixir. Tanuki was initially written in Erlang, but Elixir and Phoenix became increasingly irresistible.

### Why CouchDB

It is built with a strategy very similar to [ZFS](https://en.wikipedia.org/wiki/ZFS) in that it never overwrites live data, meaning it can crash at any time and not corrupt the data. It's schema-less, document-oriented design makes adding or changing data later very easy. But mostly it was about the incredible robustness of the application.

Of course, this all works really well if you deploy on a robust system. ZFS should be obvious, and [FreeBSD](https://www.freebsd.org) comes with ZFS built-in, and also happens to provide the latest of everything. Hence the [Vagrant](https://www.vagrantup.com) image is for FreeBSD, and most testing and deployment happens on FreeBSD.

### Why Elm

The Elm language compiler and runtime make it nearly impossible to have runtime exceptions. The language is very clean, well defined, easy to learn, and produces fast JavaScript code. Like Elixir, Elm data is immutable, leading to safer code. All actions are performed via commands to the runtime, and updates are delivered via messages to the application. This leads to an architecture that scales well with the application. The Elm "time-traveling" debugger makes finding the few mistakes that are made quite easy.

## Deploying

1. Edit the `prod.exs` in each `config` directory to modify any settings, as needed.
1. Digest the web assets (enables cache invalidation): `mix phx.digest`
1. Build the release: `mix release`
    * You will need to provide several environment variables:
        - `COOKIE` to set the cookie for connecting to the node. Must be atom-compatible.
        - `HOST` is the name of the externally visible host (e.g. `example.com`).
        - `PORT` is the number of the externally visible port, typically `80`.
1. Copy the contents of `_build/prod/rel` to the desired installation location (e.g. `/opt`).
1. Start it up, likely using `sudo`. Be sure to specify the `PORT` on which Phoenix will bind, which may be different than the externally visible port mentioned above (i.e. if Phoenix is sitting behind another web server, such as nginx).
1. Occasionally check the log files in `log` directory.

For example:

```shell
$ cd assets
$ npm install
$ node node_modules/.bin/brunch build
$ cd ..
$ MIX_ENV=prod mix phx.digest
$ COOKIE=monster HOST=example.com PORT=80 MIX_ENV=prod mix release --env=prod
$ sudo mkdir -p /opt/tanuki
$ sudo tar -C /opt/tanuki -zxf _build/prod/rel/tanuki/releases/*/tanuki.tar.gz
$ sudo PORT=8080 /opt/tanuki/bin/tanuki start
```

To test that the node has started successfully, invoke the `ping` command like so:

```shell
$ /opt/tanuki/bin/tanuki ping
pong
```

### BSD daemon

See the `config/tanuki.rc` file for an example of managing the tanuki application as a daemon via `rc.d` in BSD systems (in particular FreeBSD, and likely NetBSD as well). You will need to build and deploy the application as described above, and then use the `service` command to start it, as illustrated in the comments at the top of the file.

### Remote Console

You can open a console to the running node like so (assuming tanuki is in `/opt`):

```shell
$ /opt/tanuki/bin/tanuki remote_console
```
