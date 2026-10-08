# Use CouchDB

- Status: accepted
- Deciders: Nathan Fiedler
- Date: 2025-11-08

## Context

For the last five years the application has stored the asset records in RocksDB. However, with the massive rewrite of 2025, it became easier to rethink which data store to use for the asset records. The application originally used CouchDB and it did work quite well -- it suited the flat schema and the need to search for assets on a variety of fields.

CouchDB offers several nice features, such as a built-in web interface and multi-primary replication. It can manage multiple databases easily and is designed for reliability. The web interface offers easy access to the stored records, as well as a (primitive) method for updating individual records. It offers several methods for querying the records, the most powerful of which is their map/reduce views.

While CouchDB is not an embedded (in-process) database, with the change in overall architecture, that is no longer a concern. Deploying CouchDB is as simple as invoking a docker command and it offers a stand-alone, resilient data store that is not dependent on the application.

Regarding alternatives to CouchDB, the JavaScript version named [PouchDB](https://pouchdb.com/) has not had a release since May of 2024. It was based on [LevelDB](https://github.com/google/leveldb) and offered a very similar API to CouchDB. It was used by this project from late 2017 to early 2020 at which point it was replaced by [RocksDB](https://rocksdb.org/) and [mokuroku](https://github.com/nlfiedler/mokuroku).

## Decision

Use **CouchDB** because it is hard to beat.

## Consequences

Given that CouchDB was the database used from 2014 to 2017, it is a known and trusted data store.

## Links

- CouchDB [website](https://couchdb.apache.org/)
- couchdb-nano [GitHub](https://github.com/apache/couchdb-nano)
