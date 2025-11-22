# Rewrite Winter 2025

## Goals

### Must have

* Data storage reliability
* Access to database via CLI or browser
* Thorough support for Web APIs
* Image library and EXIF reader

### Nice to have

* GraphQL for easy access to data
* HEIC, MP4 support

## Present Limitations

* Mixing asynchronous and synchronous code can be troublesome
* Rust on the frontend is verbose and thread safety makes it cumbersome
* Rust/Leptos use of Web APIs can be difficult for some functionality (WASM integration is still lacking)
* Leptos: slow builds, beta quality, buggy libraries
* mp4: 0.14 broke support for some existing video files

## Candidates

### Database

* CouchDB
* SQLite
* DuckDB

### Backend

* Node.js
* Erlang

### Frontend

* JavaScript
* Bulma CSS

### Wire Protocol

* REST
* GraphQL

### Required Libraries

* http server
* http client (for reverse geocoding)
* web framework
* image (produce thumbnail)
* EXIF (read date/time, orientation, location)
* JSON
* ULID
* base64
* dotenv
* mime guesser
* sha2

### Erlang

#### Consideration

Erlang itself is appealing and offers fault-tolerance and scalability. Its weakness is in the dearth of 3rd party modules for functionality necessary for this project. Erlang lacks a full-stack story, requiring different languages for the front-end and backend components.

#### Libraries

* couchdb: https://github.com/benoitc/couchbeam
* http server: https://github.com/ninenines/cowboy
* http client: https://github.com/benoitc/hackney
* GraphQL: very old - https://github.com/jlouis/graphql-erlang
* EXIF: https://github.com/erlangpack/erlang_exif
* image: https://github.com/akash-akya/vix
* JSON: https://github.com/michalmuskala/jason
* mime: https://github.com/erlangpack/mimetypes
* ulid: https://github.com/TheRealReal/ecto-ulid
* base64: https://github.com/dvv/base64url
* sha2: https://github.com/diodechain/erlsha2
* dotenv: https://github.com/fireproofsocks/dotenvy

### Node.js

#### Consideration

For an application that wants to use CouchDB, GraphQL, good image libraries, and much more, it is difficult to find an ecosystem with broader support than JavaScript and Node.js. Everything is there, often with several choices, and setup and development is very fast. The client side wants to make extensive use of the Web APIs and that is best achieved with JavaScript.

#### Libraries

* couchdb: https://github.com/apache/couchdb-nano
* http server: https://expressjs.com
* http client: fetch() or https://github.com/sindresorhus/ky or https://github.com/axios/axios
* GraphQL server: https://github.com/apollographql/apollo-server
* GraphQL client: https://github.com/apollographql/apollo-client
* EXIF: https://github.com/mattiasw/ExifReader
* image: https://github.com/lovell/sharp
* mime: https://github.com/broofa/mime
* ulid: https://github.com/ulid/javascript
* base64: Node.js Buffer
* sha2: node:crypto
* in-memory cache: https://github.com/sindresorhus/quick-lru
* dotenv: https://github.com/motdotla/dotenv
* HEIC: https://github.com/catdad-experiments/libheif-js
* MP4: https://github.com/gpac/mp4box.js

### Frontend frameworks

#### Candidates

* https://vite.dev
* https://github.com/szymmis/vite-express
* https://www.solidjs.com
* https://react.dev

#### Nice to have

* Drop zone
* Light/dark switch
