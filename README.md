# Tanuki

A system for importing, storing, categorizing, browsing, displaying, and searching files, primarily images and videos. Attributes regarding the files are stored in a schema-less, document-oriented database. Designed to store millions of files. Primary interface is a web front-end with a simple design suitable for most web browsers.

Yes, another one of these. I am a programmer, it's what I do. I want to learn Erlang and this is a good project with which to learn Erlang and OTP. See the Requirements wiki for additional technical reasons.

## Current Status

The incoming processor, backend application, and web interface are written in Erlang and built using [rebar](https://github.com/rebar/rebar/) and [relx](https://github.com/erlware/relx). There is much yet to be done, mostly in the web interface.

## Building and Testing

### Prerequisites

* Erlang/OTP R17|R18
    - Homebrew: `brew install erlang`
    - FreeBSD: `pkg install erlang`
    - Ubuntu Linux: build from source as shown in the `Dockerfile`
* CouchDB 1.6.1 or higher
    - Homebrew: `brew install couchdb`
    - FreeBSD: `pkg install couchdb`
    - Ubuntu Linux: build from source as shown in the `Dockerfile`
* ImageMagick
    - See the development setup docs in the `magick-rust` project.

Once the above prerequisites are installed, some configuration may be necessary. In particular, the path to the incoming assets and the destination for the stored assets should be configured for your system. This is done for the two applications (`tanuki_backend` and `tanuki_incoming`) using a configuration file named `user_env.confg`, located in each of the application source directories (`apps/tanuki_backend` and `apps/tanuki_incoming`). The contents of these files will look something like this:

```
{assets_dir, "/Users/adam/testing/assets"}.
{incoming_dir, "/Users/adam/testing/incoming"}.
```

The full set of settings can be found in the `*.app.src.script` files in `apps/tanuki_incoming/src` and `apps/tanuki_backend/src` directories. These are processed by `rebar` at build time and effect the development and release builds. The Common Test suites are configured in the test code and hence ignore these settings.

Once the applications are configured, build the project like so:

```
$ make prepare
$ make dev
$ ./_rel/tanuki/bin/tanuki-dev
```

The web server will be listening on port 8000. Be sure to have a CouchDB instance running.

### Triggering Processing

To trigger the processing of digital assets in the "incoming" directory, without having to wait for the folders to be more than an hour old, connect to the remote node and send a message to the incoming processor, like so:

```
 erl -name foo -remsh 'tanuki@127.0.0.1'
Erlang/OTP 18 [erts-7.2.1] [source] [64-bit] [smp:8:8] [async-threads:10] [hipe] [kernel-poll:false] [dtrace]

Eshell V7.2.1  (abort with ^G)
(tanuki@127.0.0.1)1> gen_server:call(tanuki_incoming, process_now).
ok
(tanuki@127.0.0.1)2>
User switch command
 --> q
```

### Docker

If setting up the necessary prerequisites seems like too much work, there is a `Dockerfile` in the `docker` directory, which will build an Ubuntu Linux container to run tanuki. For this you will need [Docker](https://www.docker.com) installed, both to build and run the container. See the instructions at the top of the `Dockerfile` for some guidance on how to use it. If you are using Mac OS X, check out [boot2docker](http://boot2docker.io), which works very well.
