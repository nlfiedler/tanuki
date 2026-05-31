# Tanuki

An application for organizing assets, primarily images and videos. Written in [TypeScript](https://www.typescriptlang.org) and running on [Bun](https://bun.com), with a [SolidJS](https://www.solidjs.com)-powered front-end, connected via [GraphQL](https://graphql.org) and REST. Metadata is stored in either [CouchDB](https://couchdb.apache.org), [PouchDB](https://pouchdb.com), or [SQLite](https://sqlite.org/) and file content is stored unmodified within a date-time formatted directory structure, either on local disk or remotely via [namazu](https://github.com/nlfiedler/namazu).

Originally inspired by [perkeep](https://perkeep.org) as a means of organizing personal photos and videos. A key aspect of this application is that it stores all of the assets in unmodified form (no chunking or packing) in a logical directory structure. The database is used to associate tags and additional location information with assets to enable searching. The import process discards duplicates by computing a checksum and checking the database for a match. The batch editing feature allows for adding and removing tags, changing the location labels, and assigning a date-time to multiple assets.

The home and search pages offer multiple gallery display options, including Bulma cards in a flexible grid, Flickr-style justified rows, Pinterest-style masonry columns, a grid with square thumbnails and metadata that strongly resembles [PhotoPrism](https://www.photoprism.app), and a grid with smaller fixed-size thumbnails centered on the page.

## Requirements

CouchDB is only necessary for development and testing. For deployment, any supported database may be used.

- [Bun](https://bun.com)
- [CouchDB](https://couchdb.apache.org)

## Initial Setup

```bash
bun install
```

## Testing and Running

The unit tests require several environment variables, described in the **Configuration** section below. Namely, the `DATABASE_*` settings and `GOOGLE_MAPS_API_KEY` will be needed for the tests to pass successfully.

To run the automated tests, create a `test/.env` file that defines the necessary settings, making sure the values are appropriate for your setup, and then invoke `bun test` to run the test suite.

```shell
DATABASE_URL=http://192.168.1.3:5984
DATABASE_NAME=unit-tests
DATABASE_USER=admin
DATABASE_PASSWORD=admin
GOOGLE_MAPS_API_KEY=sE9tVvuPtnE6YqAIzaSyCb2V4yRAcCsEXMbyLK0
SQLITE_DBPATH=tmp/test/sqlite
POUCHDB_PATH=tmp/test/pouchdb
```

The automated tests will destroy and recreate the database(s) each time the tests are run.

To run the application locally, create a `.env.development` file with the appropriate settings and then invoke `bun start` to run the server -- it will be listening for HTTP connections on port `3000` by default. The GraphQL web interface is available at `/graphql`.

```shell
bun start            # run dev server (auto-runs codegen first)
bun test             # run all tests
bun test <path>      # run a single test file, e.g.: bun test test/shared/collections/array-deque.test.ts
bun run codegen      # regenerate GraphQL TypeScript types from schema
```

## Configuration

The application is configured using environment variables.

- **ASSETS_PATH**
  - Full path to the base directory of the asset storage, unless `NAMAZU_URL` is set.
- **UPLOAD_PATH**
  - Full path to the directory into which uploaded files will be temporarily stored.
- **DATABASE_URL**
  - URL for the CouchDB instance.
- **DATABASE_NAME**
  - Name of the CouchDB database to use for record storage.
- **DATABASE_USER**
  - Name of the CouchDB user with sufficient privileges for creating the database named in `DATABASE_NAME`.
- **DATABASE_PASSWORD**
  - Password for the CouchDB user named in `DATABASE_USER`.
- **DATABASE_HEARTBEAT_MS**
  - Frequency in milliseconds for requesting latest changes from the database in order to keep the connection alive. Default is `60000` (60 seconds).
- **FACE_CLUSTER_THRESHOLD**
  - Cosine-similarity threshold for online clustering: a detected face joins the nearest existing person when their similarity is at least this, otherwise it seeds a new person. Default is `0.5`, which is well-characterized for MobileFaceNet on aligned crops.
- **FACE_DETECT_MODEL_PATH**
  - Path to the SCRFD face-detection ONNX model. Defaults to `models/scrfd_2.5g.onnx`. Only used when `NAMAZU_URL` is not set (otherwise face detection runs on the namazu server).
- **FACE_EMBED_MODEL_PATH**
  - Path to the MobileFaceNet face-embedding ONNX model. Defaults to `models/mobilefacenet.onnx`. Only used when `NAMAZU_URL` is not set.
- **FACE_MODEL_VERSION**
  - Identifier recorded on each stored face embedding; embeddings are only ever compared within the same version. Defaults to `mobilefacenet-v1`. Override only to match a namazu deployment whose embedding model has advanced ahead of this build.
- **FACE_STORE_PATH**
  - Path and filename for the face recognition database file (SQLite). This is always needed, regardless of whether CouchDB, PouchDB, or SQLite is used for the records store (SQLite is bested suited for storing the heaps of binary data involved with face recognition).
- **GOOGLE_MAPS_API_KEY**
  - If defined, enables reverse geocoding using the Google Maps API.
- **LOG_LEVEL**
  - One of the Winston [logging levels](https://github.com/winstonjs/winston?tab=readme-ov-file#logging-levels)
- **NAMAZU_URL**
  - URL for the [namazu](https://github.com/nlfiedler/namazu) blob store. If not set, assets will be stored in `ASSETS_PATH`.
- **NODE_ENV**
  - If set to `production`, changes the logging format. Some 3rd party modules may alter their behavior slightly.
- **ORPHAN_SWEEP_INTERVAL_MS**
  - Interval in milliseconds between defensive face-store orphan sweeps, which remove face rows whose asset no longer exists. Defaults to `86400000` (24 hours); set to `0` to disable.
- **PORT**
  - Port number on which to listen for HTTP connections, defaults to `3000`.
- **POUCHDB_PATH**
  - Directory in which PouchDB will store its database files. Setting this will switch the application from using CouchDB to using PouchDB for the database (all `DATABASE_*` settings will be ignored).
- **SQLITE_DBPATH**
  - Directory in which `tanuki.sqlite` will be created, if set. Setting this will switch the application from using CouchDB to using SQLite for the database (all `DATABASE_*` settings will be ignored).
- **SYNTHETIC_CONCURRENCY**
  - Number of background workers draining the synthetic-data (labels and faces) job queue. Defaults to `2`.
- **SYNTHETIC_LOG_EVERY**
  - How many completed synthetic-data jobs between progress log lines (each reports the count processed this run and the jobs still queued). Useful for watching a backfill drain. Defaults to `100`. The same figures are also available live via the `syntheticJobStatus` GraphQL query.
- **SYNTHETIC_MODEL_PATH**
  - Path to the MobileNetV2 image-classification ONNX model. Defaults to `models/mobilenet_v2.onnx`. Only used when `NAMAZU_URL` is not set.

The application can be configured with a `.env` file thanks to Bun and [dotenv](https://github.com/motdotla/dotenv). Note, however, that during development, Bun will read this file before considering anything else, and thus it may interfere with the automated tests, which need to have tight control of the environment in order to set up mocks and spies.

As such, it is preferable to create a `.env.development` file which Bun will _not_ read when running the unit tests.

See [doc/DEPLOY.md](doc/DEPLOY.md) for deployment guidance, including the multiple stores involved, backups, and where machine-learning inference runs.

## Origin of the name

A tanuki is a racoon dog native to Japan, and may also refer to the [Bake-danuki](https://en.wikipedia.org/wiki/Bake-danuki), a shape-shifting supernatural being of Japanese folklore. That has nothing to do with this project, but the name is unique and it makes for a cute mascot.
