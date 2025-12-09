# Deploying

## Database

[CouchDB](https://couchdb.apache.org) can be deployed easily with [Docker](https://www.docker.com), just be sure to mount `/opt/couchdb/data` to a path on the host for persistent storage.

## Create database backup before upgrade

Rarely is there ever a problem with upgrades, but a backup is a good idea in general.

```shell
curl -o dump.json http://192.168.1.4:3000/records/dump
```

## Using Docker

The base directory contains a `Dockerfile` file which is used to build the application in stages and produce a relatively small final image.

On the build host:

```shell
docker build -t tanuki-app .
docker image rm 192.168.1.4:5000/tanuki
docker image tag tanuki-app 192.168.1.4:5000/tanuki
docker push 192.168.1.4:5000/tanuki
```

On the server, with a production version of a `docker-compose.yml` file that includes CouchDB as a sibling service:

```shell
docker compose down
docker compose up --build -d
```

## Geocoding Services

### Google Maps API

Google Cloud offers a [reverse geocoding](https://developers.google.com/maps/documentation/geocoding/requests-reverse-geocoding) service that is related to their Maps functionality. To get the necessary API key, follow these steps:

1. Create a Google Cloud account
1. Enable the _Geocoding API_
1. Create a new API key that is restricted to the _Geocoding API_
1. Set the `GOOGLE_MAPS_API_KEY` environment variable with the value of the API key when starting the application.

Note that the API key must be associated with the geocoding API, an existing key may work but it must be assigned to that API. A key restricted to exclusively that service is more secure against abuse.

## Database Migration

### Basic procedure

1. save various attribute counts
1. dump current database
1. stop container
1. rename database directory
1. build and deploy
1. verify empty
1. load database dump
1. verify search works
1. verify recent assets
1. compare the attribute counts
