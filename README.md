# Tanuki

An application for organizing assets, primarily images and videos. Written in [TypeScript](https://www.typescriptlang.org) with a [SolidJS](https://www.solidjs.com) powered front-end, connected via [GraphQL](https://graphql.org) and REST. Metadata is stored in [CouchDB](https://couchdb.apache.org) and file content is stored unmodified within a date/time formatted directory structure.

## Requirements

- [Bun](https://bun.com)
- [CouchDB](https://couchdb.apache.org)

## Initial Setup

```bash
bun install
```

## Testing and Running

Use `bun test` to run the test suite. The automated tests will create a database named `unit-tests`, and each time the tests are run that database will be destroyed and recreated.

Use `bun start` to run the server locally, listening for HTTP connections on port `3000` by default. The GraphQL web interface will be available at `/graphql`.

## Configuration

The application is configured using environment variables.

- **ASSETS_PATH**
  - Full path to the base directory of the asset storage.
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
- **NODE_ENV**
  - If set to `production`, changes the logging format. Some 3rd party modules may change slightly.
- **PORT**
  - Port number on which to listen for HTTP connections, defaults to `3000`.

The application can be configured with a `.env` file thanks to Bun and [dotenv](https://github.com/motdotla/dotenv). Note, however, that during development, Bun will read this file before considering anything else, and thus it may interfere with the automated tests, which need to have tight control of the environment in order to set up mocks and spies.

As such, it is preferable to create a `.env.development` file which Bun will _not_ read when running the unit tests.

## Origin of the name

A tanuki is a racoon dog native to Japan, and may also refer to the [Bake-danuki](https://en.wikipedia.org/wiki/Bake-danuki), a shape-shifting supernatural being of Japanese folklore. That has nothing to do with this project, but the name is unique and it makes for a cute mascot.
