# Tanuki

An application for organizing assets, primarily images and videos. Written in [TypeScript](https://www.typescriptlang.org) and running on [Bun](https://bun.com), with a [SolidJS](https://www.solidjs.com)-powered front-end, connected via [GraphQL](https://graphql.org) and REST. Metadata is stored in [CouchDB](https://couchdb.apache.org) or [SQLite](https://sqlite.org/) and file content is stored unmodified within a date-time formatted directory structure, either on local disk or remotely via [namazu](https://github.com/nlfiedler/namazu).

Originally inspired by [perkeep](https://perkeep.org) as a means of organizing personal photos and videos. A key aspect of this application is that it stores all of the assets in unmodified form (no chunking or packing) in a logical directory structure. The database is used to associate tags and additional location information with assets to enable searching. The import process discards duplicates by computing a checksum and checking the database for a match. The batch editing feature allows for adding and removing tags, changing the location labels, and assigning a date-time to multiple assets.

## Requirements

- [Bun](https://bun.com)
- [CouchDB](https://couchdb.apache.org) unless using SQLite

## Initial Setup

```bash
bun install
```

## Testing and Running

The unit tests require several environment variables, described in the **Configuration** section below. Namely, the `DATABASE_*` settings and `GOOGLE_MAPS_API_KEY` will be needed for the tests to pass successfully.

Use `bun test` to run the test suite. The automated tests will create a database named `unit-tests`, and each time the tests are run that database will be destroyed and recreated.

Use `bun start` to run the server locally, listening for HTTP connections on port `3000` by default. The GraphQL web interface will be available at `/graphql`.

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
- **GOOGLE_MAPS_API_KEY**'
  - If defined, enables reverse geocoding using the Google Maps API.
- **LOG_LEVEL**
  - One of the Winston [logging levels](https://github.com/winstonjs/winston?tab=readme-ov-file#logging-levels)
- **NAMAZU_URL**
  - URL for the [namazu](https://github.com/nlfiedler/namazu) blob store. If not set, assets will be stored in `ASSETS_PATH`.
- **NODE_ENV**
  - If set to `production`, changes the logging format. Some 3rd party modules may change slightly.
- **PORT**
  - Port number on which to listen for HTTP connections, defaults to `3000`.
- **SQLITE_DBPATH**
  - Directory in which `tanuki.sqlite` will be created, if set. Setting this will switch the application from using CouchDB to using SQLite for the database (all `DATABASE_*` settings will be ignored).

The application can be configured with a `.env` file thanks to Bun and [dotenv](https://github.com/motdotla/dotenv). Note, however, that during development, Bun will read this file before considering anything else, and thus it may interfere with the automated tests, which need to have tight control of the environment in order to set up mocks and spies.

As such, it is preferable to create a `.env.development` file which Bun will _not_ read when running the unit tests.

## Origin of the name

A tanuki is a racoon dog native to Japan, and may also refer to the [Bake-danuki](https://en.wikipedia.org/wiki/Bake-danuki), a shape-shifting supernatural being of Japanese folklore. That has nothing to do with this project, but the name is unique and it makes for a cute mascot.
