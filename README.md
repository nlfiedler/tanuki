# Tanuki

A system for importing, storing, categorizing, browsing, displaying, and searching files, primarily images and videos. Attributes regarding the files are stored in a schema-less, document-oriented database. Designed to store millions of files. Primary interface is a web front-end with a simple design suitable for most web browsers.

## Current Status

The incoming processor, backend application, and web interface are written in a combination of Erlang and Elixir, with everything being built using [mix](https://hexdocs.pm/mix/Mix.html). Some of the dependencies require Rust.

## Building and Testing

### Prerequisites

* [Erlang/OTP](http://www.erlang.org) R18 or higher
* [Elixir](http://elixir-lang.org) 1.3 or higher
* [CouchDB](http://couchdb.apache.org) 1.6.1 or higher
* [ImageMagick](http://www.imagemagick.org) 6.9.x
* [Rust](https://www.rust-lang.org) 1.10 or higher

#### Example for MacOS

This example asssumes you are using [Homebrew](http://brew.sh) to install the dependencies, which provides up-to-date versions of everything needed. The `xcode-select --install` is there just because the command-line tools sometimes get out of date, and some of the dependencies will fail to build without them.

```
$ xcode-select --install
$ brew install erlang
$ brew install elixir
$ brew install couchdb
$ brew install imagemagick
$ brew install rust
```

### Configuration

Once the above prerequisites are installed, you will then need to configure the various applications (`tanuki_backend` and `tanuki_incoming`) to override certain settings. To do this, write a `dev.exs` that sets the `assets_dir` and `incoming_dir` as appropriate for your system. See the examples below for some guidance.

```
$ cat apps/tanuki_backend/config/dev.exs
use Mix.Config

config :tanuki_backend,
  assets_dir: '/Users/samiam/Downloads/tanuki'

$ cat apps/tanuki_incoming/config/dev.exs
use Mix.Config

config :tanuki_incoming,
  incoming_dir: '/Users/samiam/Downloads/incoming'
```

### Commands

Once the configuration is in place, you can run build everything and run the tests like so:

```
$ mix deps.get
$ mix compile
$ mix test
```

To start an instance configured for development, run the commands shown below. While it would be nice to invoke `mix phoenix.server` from the umbrella project, Phoenix insists on trying to reload the Erlang applications, and fails. For now, just run the web app.

```
$ cd apps/tanuki_web
$ mix phoenix.server
```

The web server will be listening on port 4000. Be sure to have a CouchDB instance running.

### Triggering Processing

To trigger the processing of digital assets in the "incoming" directory, without having to wait for the folders to be more than an hour old, connect to the remote node and send a message to the incoming processor, like so:

```
$ erl -noshell -sname tanuki_in@localhost -eval "rpc:call(tanuki@localhost, gen_server, call, [tanuki_incoming, process_now]), init:stop()."
```

### Deploying

1. Write a configuration file, named `user.exs` into each `config` directory (i.e. in `tanuki_backend` and `tanuki_incoming`) to override any settings, as needed.
1. Digest the web assets (producing unique names to aid in cache invalidation): `MIX_ENV=prod mix phoenix.digest`
1. Build the release: `MIX_ENV=prod mix release --env=prod`
1. Copy the contents of `_build/prod/rel` to the desired installation location (e.g. `/opt`).
1. Start it up, likely using `sudo`.
1. Occasionally check the log files in `/opt/tanuki/log`.

For example:

```shell
$ MIX_ENV=prod mix phoenix.digest
$ MIX_ENV=prod mix release --env=prod
$ sudo mkdir -p /opt
$ sudo cp -R _build/default/rel/tanuki /opt
$ sudo /opt/tanuki/bin/tanuki -detached
```

### BSD daemon

See the `config/tanuki.rc` file for an example of managing the tanuki application as a daemon via `rc.d` in BSD systems (in particular FreeBSD, and likely NetBSD as well). You will need to build and deploy the application as described above, and then use the `service` command to start it, as illustrated in `tanuki.rc`.
