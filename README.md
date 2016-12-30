# Tanuki

A system for importing, storing, categorizing, browsing, displaying, and searching files, primarily images and videos. Attributes regarding the files are stored in a schema-less, document-oriented database. Designed to store millions of files. Primary interface is a web front-end with a simple design suitable for most web browsers.

## Current Status

The incoming processor, backend application, and web interface are written in Erlang and built using [rebar3](https://github.com/erlang/rebar3/). There is much yet to be done, mostly in the web interface.

## Building and Testing

### Prerequisites

* Erlang/OTP R17 or higher
    - Homebrew: `brew install erlang`
    - FreeBSD: `pkg install erlang`
* CouchDB 1.6.1 or higher
    - Homebrew: `brew install couchdb`
    - FreeBSD: `pkg install couchdb`
* ImageMagick
    - See the development setup docs in the `magick-rust` project.

Once the above prerequisites are installed, some configuration may be necessary. In particular, the path to the incoming assets and the destination for the stored assets should be configured for your system. This is done for the two applications (`tanuki_backend` and `tanuki_incoming`) using a single configuration file named `user_env.confg`, located at the base of the source tree. The contents of this file will look something like this:

```
{assets_dir, "/Users/adam/testing/assets"}.
{incoming_dir, "/Users/adam/testing/incoming"}.
```

The full set of settings can be found in the `*.app.src.script` files in the `apps/tanuki_backend/src` and `apps/tanuki_incoming/src` directories. These are processed by `rebar3` at build time and affect the development and release builds. The Common Test suites are configured in the test code and hence ignore these settings.

Once the applications are configured, build and test the project like so:

```
$ rebar3 ct
```

To start an instance configured for development, use the following commands:

```
$ rebar3 release
$ ./_build/default/rel/tanuki/bin/tanuki-dev
```

The web server will be listening on port 8000. Be sure to have a CouchDB instance running.

### Triggering Processing

To trigger the processing of digital assets in the "incoming" directory, without having to wait for the folders to be more than an hour old, connect to the remote node and send a message to the incoming processor, like so:

```
$ erl -noshell -sname tanuki_in@localhost -eval "rpc:call(tanuki@localhost, gen_server, call, [tanuki_incoming, process_now]), init:stop()."
```

### Deploying

1. Write a configuration file, named `user_env.config`, at the base of the source tree.
1. Build the release: `rebar3 as prod release`
1. Copy the contents of `_build/default/rel` to the desired installation location (e.g. `/opt`).
1. Start it up, likely using `sudo`.
1. Occasionally check the log files in `/opt/tanuki/log`.

For example:

```shell
$ cp ~/tanuki_custom.config user_env.config
$ rebar3 as prod release
$ sudo mkdir -p /opt
$ sudo cp -R _build/default/rel/tanuki /opt
$ sudo /opt/tanuki/bin/tanuki -detached
```

### BSD daemon

See the `config/tanuki.rc` file for an example of managing the tanuki application as a daemon via `rc.d` in BSD systems (in particular FreeBSD, and likely NetBSD as well). You will need to build and deploy the application as described above, and then use the `service` command to start it, as illustrated in `tanuki.rc`.
