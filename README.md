# Tanuki

A system for importing, storing, categorizing, browsing, displaying, and searching files, primarily images and videos. Attributes regarding the files are stored in a schema-less, document-oriented database. Designed to store millions of files. Primary interface is a web front-end with a simple design suitable for most web browsers.

Yes, another one of these. I am a programmer, it's what I do. I want to learn Erlang and this is a good project with which to learn Erlang and OTP. See the Requirements wiki for additional technical reasons.

## Current Status

The incoming processor, backend application, and web interface are written in Erlang and built using [rebar](https://github.com/rebar/rebar/) and [relx](https://github.com/erlware/relx). There is much yet to be done, mostly in the web interface.

## Building and Testing

Install Erlang/OTP R17 and CouchDB, then build the project like so:

```
$ rebar get-deps
$ rebar -r prepare-deps
$ make dev
$ ./_rel/tanuki/bin/tanuki-dev
```

The web server will be listening on port 8000. Be sure to have a CouchDB instance running.
