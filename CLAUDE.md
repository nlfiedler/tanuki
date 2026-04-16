# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

Tanuki is a digital asset management application for organizing images, videos, and other files. It stores assets unmodified in a date-time directory structure, with metadata (tags, location, dates) in a pluggable database. The server is TypeScript on Bun with Express + Apollo GraphQL; the client is SolidJS.

## Commands

```bash
bun install          # install dependencies
bun start            # run dev server (auto-runs codegen first)
bun test             # run all tests
bun test <path>      # run a single test file, e.g.: bun test test/shared/collections/array-deque.test.ts
bun run codegen      # regenerate GraphQL TypeScript types from schema
```

Tests require a `test/.env` file with database credentials and `GOOGLE_MAPS_API_KEY`. The test suite destroys and recreates databases on each run.

For local development, create `.env.development` (not `.env` — that interferes with tests) with the required environment variables.

## Architecture

The server follows a clean architecture with three layers:

**Presentation** (`server/preso/`) — Express routes + Apollo GraphQL server. The GraphQL schema lives in `server/preso/graphql/schema.graphql`. REST endpoints handle file upload/download for assets. The GraphQL playground is at `/graphql`.

**Domain** (`server/domain/`) — Business logic organized as ~20 use cases (`GetAsset`, `ImportAsset`, `SearchAssets`, `EditAssets`, `ScanAssets`, etc.). Entities include Asset, Location, and Search. Use cases receive their dependencies via constructor injection.

**Data** (`server/data/repositories/`) — Pluggable repositories. Database: CouchDB (default), PouchDB (set `POUCHDB_PATH`), or SQLite (set `SQLITE_DBPATH`). Blob storage: local filesystem (`ASSETS_PATH`) or remote Namazu (`NAMAZU_URL`). Location: Google Maps reverse geocoding if `GOOGLE_MAPS_API_KEY` is set, otherwise a dummy stub.

**Dependency Injection** (`server/container.ts`) — Awilix wires everything together. This is where the active repository implementations are selected based on environment variables.

**Client** (`client/`) — SolidJS SPA using Apollo Client for GraphQL. Routes map to pages (Home, Search, Upload, Pending, Browse, AssetDetails, Edit). Styled with Bulma + SCSS. GraphQL TypeScript types are generated into `generated/`.

## Key Configuration

| Variable | Purpose |
|---|---|
| `ASSETS_PATH` | Base directory for local asset storage |
| `UPLOAD_PATH` | Temp directory for file uploads |
| `DATABASE_URL/NAME/USER/PASSWORD` | CouchDB connection |
| `POUCHDB_PATH` | Switches to PouchDB (ignores `DATABASE_*`) |
| `SQLITE_DBPATH` | Switches to SQLite (ignores `DATABASE_*`) |
| `NAMAZU_URL` | Switches to remote blob store |
| `GOOGLE_MAPS_API_KEY` | Enables reverse geocoding |
