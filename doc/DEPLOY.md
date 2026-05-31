# Deploying

## Stores

Regardless of which records backend is configured (CouchDB, PouchDB, or SQLite), face and person data always live in a separate SQLite database at `FACE_STORE_PATH`. A typical deployment therefore spans more than one store: the records database, a blob store for file content (local `ASSETS_PATH` or a remote [namazu](https://github.com/nlfiedler/namazu) server via `NAMAZU_URL`), and the face store. In a container, point `FACE_STORE_PATH` at a mounted volume so the face database persists across restarts.

## Database

[CouchDB](https://couchdb.apache.org) can be deployed easily with [Docker](https://www.docker.com), just be sure to mount `/opt/couchdb/data` to a path on the host for persistent storage.

## Create database backup before upgrade

Rarely is there ever a problem with upgrades, but a backup is a good idea in general.

```shell
curl -o dump.json http://192.168.1.4:3000/records/dump
```

The face recognition database is always stored in SQLite files located at the path given by the `FACE_STORE_PATH` environment variable. Those files should also be saved since they are _not_ part of the dump and load procedure. If this database is ever lost, you can run the `backfillFaceRecognition` GraphQL mutation via the GraphQL playground (at the `/graphql` route).

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

### Restarting and redeploying safely

The background synthetic-data work (label and face extraction) is queued in the
face store's SQLite database, so as long as `FACE_STORE_PATH` is on a persistent
volume (see [Stores](#stores)), a stop/redeploy does not lose progress:

- **Completed work is durable.** Each asset's results are committed to the face
  store as the job finishes.
- **The queue resumes on its own.** On startup the worker pool drains whatever
  jobs remain in `synthetic_jobs`; you do _not_ need to re-run a backfill for the
  jobs still in the queue.
- **In-flight jobs are drained gracefully.** On `SIGTERM` the server stops
  claiming new jobs and waits for the one or two currently processing to finish
  and persist before exiting.

The only caveat is the shutdown grace period. `docker stop` (and
`docker compose down`) send `SIGTERM` and then `SIGKILL` after **10 seconds** by
default. Face inference is normally well under a second per image, but a large
image or a slow `NAMAZU_URL` round-trip could exceed that window, in which case
the kill drops up to `SYNTHETIC_CONCURRENCY` (default 2) in-flight assets back to
`PENDING` (not corrupted, just skipped). To give the drain room to complete:

```shell
docker stop -t 60 <container>     # 60s grace instead of 10s
```

For Compose, set `stop_grace_period: 60s` on the service instead.

After the new container is up, you can optionally run the
`backfillFaceRecognition` mutation once via the GraphQL playground (`/graphql`).
It is idempotent — it re-enqueues only the handful of assets (if any) that were
mid-flight at shutdown and skips everything already processed. To watch a
backfill drain, query `syntheticJobStatus` (also surfaced as a progress banner on
the Labels and People pages) or follow the worker pool's periodic progress log
lines (cadence set by `SYNTHETIC_LOG_EVERY`).

## Machine learning inference

Image classification (labels) and face recognition run on machine-learning
models. When `NAMAZU_URL` is set, inference is pushed to the namazu server's
`/synthetic` endpoint, so the tanuki host needs no model files or
`onnxruntime-node` native dependencies. Otherwise the models run in-process on
the tanuki host: they are fetched into `models/` per the model manifest, and the
host must satisfy the native requirements of `onnxruntime-node`.

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
