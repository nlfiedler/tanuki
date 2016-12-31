# Tanuki

A system for importing, storing, categorizing, browsing, displaying, and searching files, primarily images and videos. Attributes regarding the files are stored in a schema-less, document-oriented database. Designed to store millions of files. Primary interface is a web front-end with a simple design suitable for most web browsers.

## Current Status

The incoming processor, backend application, and web interface are written in a combination of Erlang and Elixir, with everything being built using [mix](https://hexdocs.pm/mix/Mix.html).

## Building and Testing

### Prerequisites

* Erlang/OTP R18 or higher
    - Homebrew: `brew install erlang`
    - FreeBSD: `pkg install erlang`
* Elixir 1.3 or higher
    - Homebrew: `brew install elixir`
* CouchDB 1.6.1 or higher
    - Homebrew: `brew install couchdb`
    - FreeBSD: `pkg install couchdb`
* ImageMagick
    - See the development setup docs in the `magick-rust` project.

Once the above prerequisites are installed, run the tests like so:

```
$ mix deps.get
$ mix compile
$ mix ct
```

To start an instance configured for development, use the following commands:

```
$ mix release
$ _build/dev/rel/tanuki/bin/tanuki foreground
```

The web server will be listening on port 8000. Be sure to have a CouchDB instance running.

### Triggering Processing

To trigger the processing of digital assets in the "incoming" directory, without having to wait for the folders to be more than an hour old, connect to the remote node and send a message to the incoming processor, like so:

```
$ erl -noshell -sname tanuki_in@localhost -eval "rpc:call(tanuki@localhost, gen_server, call, [tanuki_incoming, process_now]), init:stop()."
```

### Deploying

1. Write a configuration file, named `user.exs` into each `config` directory (i.e. in `tanuki_backend` and `tanuki_incoming`) to override any settings, as needed.
1. Build the release: `mix release --env=prod`
1. Copy the contents of `_build/prod/rel` to the desired installation location (e.g. `/opt`).
1. Start it up, likely using `sudo`.
1. Occasionally check the log files in `/opt/tanuki/log`.

For example:

```shell
$ mix release --env=prod
$ sudo mkdir -p /opt
$ sudo cp -R _build/default/rel/tanuki /opt
$ sudo /opt/tanuki/bin/tanuki -detached
```

### BSD daemon

See the `config/tanuki.rc` file for an example of managing the tanuki application as a daemon via `rc.d` in BSD systems (in particular FreeBSD, and likely NetBSD as well). You will need to build and deploy the application as described above, and then use the `service` command to start it, as illustrated in `tanuki.rc`.
