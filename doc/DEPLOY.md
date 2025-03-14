# Deploying

## Database

By default the application will use [RocksDB](https://rocksdb.org) for storing metadata, but can be configured to use [SQLite](https://sqlite.org) instead. This is done by setting the `DATABASE_TYPE` environment variable to the value "sqlite" (compared case-insensitively, so "SQLite" will also work).

## Using Docker

The base directory contains a `docker-compose.yml` file which is used to build the application in stages and produce a relatively small final image.

On the build host:

```shell
docker compose build --pull
docker image rm 192.168.1.11:5000/tanuki
docker image tag tanuki-app 192.168.1.11:5000/tanuki
docker push 192.168.1.11:5000/tanuki
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
