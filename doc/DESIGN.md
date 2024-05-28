# Architecture and Design

Assets stored unmodified within a date/time formatted directory structure, metadata stored in a key-value store, an HTTP server backend, and a single-page application for a frontend. Backend written in [Rust](https://www.rust-lang.org), front-end written in [Flutter](https://flutter.dev), data store is [RocksDB](https://rocksdb.org), wire protocol is [GraphQL](https://graphql.org) and basic HTTP API.

## Clean Architecture

The application is designed using the [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html) in which the application is divided into three layers: domain, data, and presentation. The domain layer defines the "policy" or business logic of the application, consisting of entities, use cases, and repositories. The data layer is the interface to the underlying system, defining the data models that are ultimately stored in a database. The presentation layer is what the user generally sees, the web interface, and to some extent, the GraphQL interface.

## Storage

When an asset is added to the system, several steps are performed:

1. A hash digest of the file is computed to identify duplicates.
1. An identifier based on the import date/time, filename extension,
   and [ULID](https://github.com/ulid/spec) is generated.
1. A minimal document, with checksum and identifier, is stored in the database.
1. The asset is stored in a directory structure according to import date/time.
    - e.g. `2017/05/13/1630/01ce0d526z6cyzgm02ap0jv281.jpg`
    - the minutes are rounded to `:00`, `:15`, `:30`, or `:45`
    - the base filename is the ULID
    - the original file extension is retained

In previous versions of the application, assets were stored in a directory structure reflecting the checksum of the asset, similar to the object store in Git. There were two directory levels consisting of two pairs of leading digits from the checksum, and the filename was the remainder of the checksum.

### Benefits

* Assets are stored in directory structure reflecting time and order of addition
    - ULID sorts by time, so order of insertion is retained
* Number of directories and files at any particular level is reasonable
    - at most 96 directories per calendar day
    - files per directory limited to what can be processed within 15 minutes
* Can rebuild some of the metadata from the directory structure and file names
    - import date/time from file path
    - media type from extension
    - original date/time from file metadata
* Encoded path as asset ID allows serving asset without database lookup
    - base64 encoded asset path happens to be same length as SHA256
* Avoids filename collisions
    - names like `IMG_1234.JPG` easily collide with other devices

### Drawbacks

The files are renamed, which might be a bother to some people. In many cases, the file names are largely irrelevant, as most are of the form `IMG_1234.JPG`. In other cases, the names are something ridiculous, like `20150419171116-63EK7JXWKEVMDJVV-P1510081.jpg`, which encodes a date/time and some seemingly random sequence of letters and numbers. The good news is the original file name is recorded in the database.
