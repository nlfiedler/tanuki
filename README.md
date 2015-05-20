# Tanuki

A system for importing, storing, categorizing, browsing, displaying, and searching files, primarily images and videos. Attributes regarding the files are stored in a schema-less, document-oriented database. Designed to store millions of files. Primary interface is a web front-end with a simple design suitable for most web browsers.

Yes, another one of these. I am a programmer, it's what I do. I want to learn Erlang and this is a good project with which to learn Erlang and OTP. See the Requirements wiki for additional technical reasons.

## Current Status

The incoming processor, backend application, and web interface are written in Erlang and built using [rebar](https://github.com/rebar/rebar/) and [relx](https://github.com/erlware/relx). There is much yet to be done, mostly in the web interface.

## Building and Testing

### Prerequisites

* Erlang/OTP R17
    - Homebrew: `brew install erlang`
    - FreeBSD: `pkg install erlang`
    - Ubuntu Linux: build from source as shown in the `Dockerfile`
* CouchDB 1.6.1 or higher
    - Homebrew: `brew install couchdb`
    - FreeBSD: `pkg install couchdb`
    - Ubuntu Linux: build from source as shown in the `Dockerfile`
* ImageMagick
    - Homebrew: `brew install imagemagick`
    - FreeBSD: `pkg install ImageMagick-nox11`
    - Ubuntu Linux: `apt-get install imagemagick`

Once the above prerequisites are installed, build the project like so:

```
$ rebar get-deps
$ rebar -r prepare-deps
$ make dev
$ ./_rel/tanuki/bin/tanuki-dev
```

The web server will be listening on port 8000. Be sure to have a CouchDB instance running.

### Docker

If setting up the necessary prerequisites seems like too much work, there is a `Dockerfile` in the `docker` directory, which will build an Ubuntu Linux container to run tanuki. For this you will need [Docker](https://www.docker.com) installed, both to build and run the container. See the instructions at the top of the `Dockerfile` for some guidance on how to use it. If you are using Mac OS X, check out [boot2docker](http://boot2docker.io), which works very well.
