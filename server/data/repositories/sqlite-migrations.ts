//
// Copyright (c) 2026 Nathan Fiedler
//
import { Database } from 'bun:sqlite';

type Migration = {
  version: number;
  up: (db: Database) => void;
};

/**
 * Ordered list of database migrations. Each migration's `up` function is run
 * exactly once, in `version` order, on databases that have not yet recorded
 * that version in `schema_migrations`.
 *
 * Add new migrations to the end; never edit or reorder existing entries.
 */
const MIGRATIONS: Migration[] = [
  {
    version: 1,
    up: (db: Database) => {
      // Baseline: the existing `assets` table, indices, and views are created
      // by `initialize()` with `CREATE TABLE IF NOT EXISTS`, so this migration
      // exists only to record that pre-migration databases are at version 1.
    }
  },
  {
    version: 2,
    up: (db: Database) => {
      db.run(
        `CREATE TABLE IF NOT EXISTS metadata (
          asset_id TEXT NOT NULL PRIMARY KEY REFERENCES assets(key) ON DELETE CASCADE,
          camera_make TEXT,
          camera_model TEXT,
          lens_make TEXT,
          lens_model TEXT,
          exposure_time TEXT,
          f_number REAL,
          iso INTEGER,
          focal_length_35mm REAL,
          original_date_offset TEXT,
          gps_latitude REAL,
          gps_longitude REAL,
          display_width INTEGER,
          display_height INTEGER,
          duration REAL,
          frame_rate REAL,
          video_codec TEXT,
          raw TEXT
        ) STRICT`
      );
    }
  },
  {
    version: 3,
    up: (db: Database) => {
      db.run(
        `CREATE TABLE IF NOT EXISTS synthetic_data (
          asset_id TEXT NOT NULL PRIMARY KEY REFERENCES assets(key) ON DELETE CASCADE,
          primary_label TEXT,
          labels TEXT,
          status TEXT NOT NULL DEFAULT 'PENDING',
          updated_at INTEGER NOT NULL
        ) STRICT`
      );
      db.run(
        `CREATE INDEX IF NOT EXISTS sd_primary_label
           ON synthetic_data(primary_label)`
      );
    }
  }
];

/**
 * Apply any pending migrations to the given database. Idempotent: safe to call
 * on every startup.
 */
export function runMigrations(db: Database): void {
  db.run(
    `CREATE TABLE IF NOT EXISTS schema_migrations (
      version INTEGER NOT NULL PRIMARY KEY,
      applied_at INTEGER NOT NULL
    ) STRICT`
  );
  const applied = new Set<number>();
  for (const row of db.query('SELECT version FROM schema_migrations')) {
    applied.add((row as any).version);
  }
  const insert = db.query(
    'INSERT INTO schema_migrations (version, applied_at) VALUES (?, ?)'
  );
  for (const migration of MIGRATIONS) {
    if (applied.has(migration.version)) continue;
    db.transaction(() => {
      migration.up(db);
      insert.run(migration.version, Math.trunc(Date.now() / 1000));
    })();
  }
}
