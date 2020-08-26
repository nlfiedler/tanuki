# Use Rust for Backend

* Status: accepted
* Deciders: Nathan Fiedler
* Date: 2020-08-20

## Context

The application consists of two parts, the server-side backend and the client-side frontend. With regards to the backend, the programming language and runtime has a significant impact on the performance and ease of deployment. Candidate languages should ideally not involve a heavyweight runtime, or the runtime should at least be sufficiently fast as to not affect the user experience. Of lesser importance is the availability of permissively licensed libraries for that language, and to some extent the tooling support.

With respect to third party libraries, of particular important is an embedded database, a web framework, and a GraphQL backend. These represent rather significant amounts of work that would be best not taken on within the scope of this application.

However, of utmost importance is that the application compiles to machine code. Shipping source code to the customer only invites them to pirate the work and produce derivatives.

Sensible candidates include [Go](https://golang.org) and [Rust](https://www.rust-lang.org). The former involves a non-trivial runtime with a memory manager that is somewhat slow in benchmarks. Rust, on the other hand, has barely any runtime to speak of, meaning that the application code compiles to raw machine code with very little overhead. Both options have plenty of available libraries, most of which are permissively licensed. In terms of popularity, Go beats Rust as there are many available libraries. That popularity, however, is slowly changing, as developers are becoming aware of the benefits Rust. For five years straight, Rust was voted the "most loved" programming language in the Stack Overflow Developer Survey.

One distintive difference with Rust when compared to Go, is that it tries very hard to be safe. Unlike Go, Rust does not have a `null` concept at all. Instead, there is a type named `Option` to represent "maybe" values, which forces the developer to deal with the possibilitly of a missing value. Similarly, errors are represented with a type that is either "ok" or "error", again forcing the developer to deal with errors explicitly. Macros and syntactic sugar make error handling lightweight as a single character, allowing most code to pass-the-buck up the call stack, presumably where it can be handled in a sensible manner. Compare this to Go, which is more boilerplate than actual application logic, as many blog posts have covered elsewhere.

Other options that were used in the past include [Dart](https://dart.dev), [Node.js](https://nodejs.org/en/) and [Erlang](https://www.erlang.org). While Dart is attractive, it is not necessarily all that fast (yet). The web framework (Angel) and GraphQL framework (also Angel) was particularly slow and incomplete. With Node, the application deliverable is the source code, so that fails the most important requirement. With Erlang, the runtime is rather large and not all that fast in terms of raw performance.

## Decision

The choice is **Rust**. It has very good tooling support, compiles to fast code, and offers a rich type system that gratifies the conscientious developer. While Go is easy to learn, it is a bit too simple and can be frustrating to maintain large-scale applications.

Additionally, Rust has not just any, but _the_ fastest web framework, [actix-web](https://actix.rs), a GraphQL backend, [Juniper](https://graphql-rust.github.io), and a very fast embedded key-value store, [RocksDB](https://rocksdb.org), via the Rust wrapper [rust-rocksdb](https://github.com/rust-rocksdb/rust-rocksdb).

## Consequences

Since May of 2020, the application backend has been written in Rust. In that time there have been zero issues with that decision. The application is fast, compiles to a self-contained, relatively small binary that is easily deployed.

## Links

* Rust [website](https://www.rust-lang.org)
* Stack Overflow [2020 survey](https://insights.stackoverflow.com/survey/2020)
