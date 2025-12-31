# Architecture and Design

Assets are stored unmodified within a date-time formatted directory structure and metadata is stored in [CouchDB](https://couchdb.apache.org/). The backend is written in [TypeScript](https://www.typescriptlang.org/) and runs on [Bun](https://bun.com/). The front-end is TypeScript with [SolidJS](https://solidjs.com), which communicates with the backend via a combination of REST and [GraphQL](https://graphql.org).

## Clean Architecture

The application is designed using the [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html) in which the application is divided into three layers: domain, data, and presentation. The domain layer defines the "policy" or business logic of the application, consisting of entities, use cases, and repositories. The data layer is the interface to the underlying system, defining the data models that are ultimately stored in a database. The presentation layer is what the user generally sees, the web interface, and to some extent, the GraphQL interface.

## Asset Storage and Asset Identifiers

When an asset is added to the system, several steps are performed:

1. A hash digest of the file is computed to identify duplicates.
1. An identifier based on the import date-time, media type, and a [ULID](https://github.com/ulid/spec) is generated.
1. A minimal document, with checksum and identifier, is stored in the database.
1. The asset is stored in a directory structure according to import date-time.
   - e.g. `2017/05/13/1630/01ce0d526z6cyzgm02ap0jv281.jpg`
   - the minutes are rounded to `:00`, `:15`, `:30`, or `:45`
   - the base filename is the ULID
   - an appropriate extension is added

In previous versions of the application, assets were stored in a directory structure reflecting the checksum of the asset, similar to the object store in Git. There were two directory levels consisting of two pairs of leading digits from the checksum, and the filename was the remainder of the checksum.

### Benefits

- Thumbnails can be served without reading from the database since the identifier is the path to the file.
- Assets are stored in directory structure reflecting time and order of addition
  - ULID sorts by time, so order of import is retained
- Number of directories and files at any particular level is reasonable
  - at most 96 directories per calendar day
  - files per directory limited to what can be processed within 15 minutes
- Can rebuild some of the metadata from the directory structure and file names
  - import date-time from file path
  - media type from extension
- Avoids filename collisions
  - names like `IMG_1234.JPG` easily collide with files from other sources

### Drawbacks

The generated identifiers are long, averaging about 64 characters (`MjAyNS8xMS8yOS8yMzAwLzAxa2I5c2Y2NmEwOHgxanZ4M2pjZ2E0amZqLmpwZw==` is a typical value), which is much longer than a ULID (16 bytes in base-32 encoded form is 26 characters). This consumes more disk space in the B-tree database in both the primary table and in the secondary indices. The value is time-oriented which results in new records being added to the end of the database, which is generally good. Aside from the length, the keys are adequate considering the anticipated number of database records (tens of thousands versus millions).

The files are renamed, which might be a bother to some people. In many cases, the file names are largely irrelevant, as most are of the form `IMG_1234.JPG`. In other cases, the names are something ridiculous, like `20150419171116-63EK7JXWKEVMDJVV-P1510081.jpg`, which encodes a date-time and some seemingly random sequence of letters and numbers. The good news is the original file name is recorded in the database.
