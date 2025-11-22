# Use JavaScript for the backend

* Status: accepted
* Deciders: Nathan Fiedler
* Date: 2025-11-14

## Context

With the massive rewrite of 2025, the opportunity to revisit the choice of language and runtime for the application presented itself yet again. Orignally, after the initial Python ingestion script, the application was written in [Erlang](https://www.erlang.org). While Erlang is really neat and works well as the backend for reliable, fault-tolerant systems, it does not have an ecosystem that supplies all of the necessary libraries that this application needs (image support, GraphQL, web framework, etc).

[JavaScript](https://en.wikipedia.org/wiki/JavaScript), on the other hand, has an enormous ecosystem and plethora of backend and front-end frameworks to choose from. In terms of GraphQL support, the [reference implemenation](https://graphql.org) and the best library, [Apollo](https://www.apollographql.com), are both written in JavaScript. Additionally, if the preferred data store is [CouchDB](https://couchdb.apache.org/) then an official client library is provided for JavaScript in the [nano](https://github.com/apache/couchdb-nano) package. It provides all of the functions necessary to manage and search a CouchDB instance.

In the first half of 2019, an effort was made to use [TypeScript](https://www.typescriptlang.org) to write the backend code. However, with the lack of a coherent design, the language offered little benefit. Fast forward 6 years and now the application is designed using Clean Architecture, for which a strongly-typed language with support for interfaces provides a lot of value. TypeScript/JavaScript make dependency injection more feasible than Rust ever will, as well as support for mocks and spies in unit tests. A strongly-typed compiled language such as TypeScript helps to catch mistakes earlier than with plain JavaScript.

With regards to the server runtime, the choices are [Node.js](https://nodejs.org/en), [Deno](https://deno.com), and [Bun](https://bun.com). All three can run JavaScript natively, while only Deno and Bun can run TypeScript without the addition of extra tooling. In particular, serving and testing a TypeScript application with Node is rather difficult, even in 2025. Based on the features and a detailed [comparison](https://betterstack.com/community/guides/scaling-nodejs/nodejs-vs-deno-vs-bun/), Bun comes out ahead as the fastest and most featureful of the offerings. Bun is a runtime, a package manager, a bundler, and a test runner (with support for mocks).

## Decision

Choose **TypeScript** and **Bun** for the backend.

## Consequences

TBD

## Links

* Bun [website](https://bun.com)
* TypeScript [website](https://www.typescriptlang.org)
