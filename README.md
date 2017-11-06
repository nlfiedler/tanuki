# Tanuki

A system for importing, storing, categorizing, browsing, displaying, and searching files, primarily images and videos. Attributes regarding the files are stored in a schema-less, document-oriented database. Designed to store millions of files. Provides a simple web interface with basic browsing and editing capabilities.

## Building and Testing

### Prerequisites

* [Node.js](https://nodejs.org/) 8.x or higher
* [Elm](http://elm-lang.org) 0.18 or higher

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

A bunch of JavaScript running on Node.js, and probably some native code in there somewhere.

### Why Node.js

That is a long story. This application started as a set of Python scripts, then became an Erlang/Nitrogen application, then was rewritten in Elixir on Phoenix. Then, due to changing requirements, was rewritten again in JavaScript on Node.js. The Elm code from the previous generation is the only part that did not change.

### Why Elm

The Elm language compiler and runtime make it nearly impossible to have runtime exceptions. The language is very clean, well defined, easy to learn, and produces fast JavaScript code. Elm data is immutable, leading to safer code. All actions are performed via commands to the runtime, and updates are delivered via messages to the application. This leads to an architecture that scales well with the application. The Elm "time-traveling" debugger makes finding the few mistakes that are made quite easy.
