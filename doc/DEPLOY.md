# Deploying

## Database

By default the application will use [RocksDB](https://rocksdb.org) for storing metadata, but can be configured to use [DuckDB](https://duckdb.org) or [SQLite](https://sqlite.org) instead. This is done by setting the `DATABASE_TYPE` environment variable to the value "duckdb" or "sqlite" (compared case-insensitively, so "DuckDB" and "SQLite" will also work).

## Database Comparison

Listed in the order in which support was added to the application.

### RocksDB

This application has used RocksDB the longest and thus it has received the most testing. In terms of write performance, little will beat a log-structured merge-tree like RocksDB. With RocksDB, this application can ingest 20,000 records in 4 seconds. However, it soaks up disk space in order to make that possible.

Another drawback of RocksDB is that there is no command-line interface. Even if it did, the data does not have a schema so examining the records would be very difficult as everything is written using [CBOR](https://cbor.io).

### SQLite

The small, fast, and reliable in-process SQL database engine. It is extremely widely used and thoroughly tested. It uses less disk space than RocksDB and offers a command-line interface. Since the data is stored in a relational database, examining and modifying the data is much easier.

The one drawback is that ingesting tens of thousands of records will take several minutes compared to a few seconds with RocksDB. That is to be expected since a relational database needs to maintain constraints and indexes while in the process of inserting or updating records.

### DuckDB

DuckDB, like SQLite, is an in-process relational database engine. Unlike SQLite, DuckDB is suited to an analytical workload rather than transactional (OLAP instead of OLTP). Like SQLite, DuckDB offers a command-line interface and structured data store. Disk usage is also very similar to SQLite.

One drawback of DuckDB is that, like SQLite, ingesting many records will be slow when compared to RocksDB.

## Using Docker

The base directory contains a `docker-compose.yml` file which is used to build the application in stages and produce a relatively small final image.

On the build host:

```shell
docker compose build --pull
docker image rm 192.168.50.201:5000/tanuki
docker image tag tanuki-app 192.168.50.201:5000/tanuki
docker push 192.168.50.201:5000/tanuki
```

On the server, with a production version of the `docker-compose.yml` file:

```shell
docker compose down
docker compose up --build -d
```

## Geocoding Services

### Google Maps API

Google Cloud offers a [reverse geocoding](https://developers.google.com/maps/documentation/geocoding/requests-reverse-geocoding) service that is related to their Maps functionality. To get the necessary API key, follow these steps:

1. Create a Google Cloud account
1. Enable the *Geocoding API*
1. Create a new API key that is restricted to the *Geocoding API*
1. Set the `GOOGLE_MAPS_API_KEY` environment variable with the value of the API key when starting the application.

Note that the API key must be associated with the geocoding API, an existing key may work but it must be assigned to that API. A key restricted to exclusively that service is more secure against abuse.

## Database Migration

### Basic procedure

1. dump current database
2. stop container
3. rename database directory
4. build and deploy
5. verify empty
6. load database dump
7. verify search works
8. verify recent assets
