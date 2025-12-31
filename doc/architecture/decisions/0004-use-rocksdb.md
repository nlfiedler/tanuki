# Use RocksDB

- Status: accepted
- Deciders: Nathan Fiedler
- Date: 2020-08-20

## Context

The application needs to store not only digital assets, but information about those assets. Given the data model is fairly flat, a simple key-value store should be adequate. What's more, for the purpose of an application designed to run on the desktop computer, the data store should be embedded (i.e. runs in-process).

## Decision

There are surprisingly few choices when it comes to an embedded database that is accessible from an application written in Rust. One is [SQLite](https://sqlite.org/index.html) which is a relational database that saves everything to a single (ever growing) file. Another is RocksDB, which itself is not written in Rust, but there is a well-maintained Rust wrapper. RocksDB is fast and actively maintained by Facebook.

The choice is **RocksDB**. It is resilient to data-loss, fast, and space efficient for the purpose of this application, which does not have relational data.

## Consequences

RocksDB has been used by this application and another for several months. It has been working well without any issues whatsoever.

RocksDB support was dropped with the major (TypeScript/Bun) rewrite at the end of 2025. A good run of five years.

## Links

- RocksDB [website](https://rocksdb.org)
- rust-rocksdb [GitHub](https://github.com/rust-rocksdb/rust-rocksdb)
