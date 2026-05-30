//
// Copyright (c) 2026 Nathan Fiedler
//
import assert from 'node:assert';
import fs from 'node:fs/promises';
import path from 'node:path';
import { Database } from 'bun:sqlite';
import {
  type JobKind,
  Person,
  SyntheticJob
} from 'tanuki/server/domain/entities/face.ts';
import { type FaceStore } from 'tanuki/server/domain/repositories/face-store.ts';
import { type SettingsRepository } from 'tanuki/server/domain/repositories/settings-repository.ts';

/** Current epoch seconds, matching the integer-time convention used elsewhere. */
function now(): number {
  return Math.trunc(Date.now() / 1000);
}

interface JobRow {
  id: number;
  asset_id: string;
  kind: string;
  priority: number;
  attempts: number;
  last_error: string | null;
  enqueued_at: number;
}

function jobFromRow(row: JobRow): SyntheticJob {
  return new SyntheticJob(
    row.id,
    row.asset_id,
    row.kind as JobKind,
    row.priority,
    row.attempts,
    row.last_error,
    row.enqueued_at
  );
}

/**
 * SQLite-backed {@link FaceStore}. Lives in its own database file, separate
 * from any asset store, so the same code path serves CouchDB, PouchDB, and
 * SQLite deployments alike.
 */
class SqliteFaceStore implements FaceStore {
  dbpath: string;
  database: Database | null;

  constructor({
    settingsRepository
  }: {
    settingsRepository: SettingsRepository;
  }) {
    const basepath = settingsRepository.get('FACE_STORE_PATH');
    assert.ok(basepath, 'missing FACE_STORE_PATH environment variable');
    this.dbpath = path.join(basepath, 'faces.sqlite');
    this.database = null;
  }

  /**
   * Destroy and recreate the database from scratch.
   *
   * @throws if `NODE_ENV` is set to 'production'.
   */
  async destroyAndCreate(): Promise<void> {
    assert.notStrictEqual(
      process.env['NODE_ENV'],
      'production',
      'destroyAndCreate() called in production!'
    );
    if (this.database) {
      this.database.close(true);
      this.database = null;
    }
    // Remove the main database file plus the WAL sidecars; leaving a stale
    // -wal/-shm behind makes a fresh connection fail with SQLITE_IOERR.
    for (const suffix of ['', '-wal', '-shm']) {
      try {
        await fs.unlink(this.dbpath + suffix);
      } catch (error: any) {
        if (error.code !== 'ENOENT') {
          throw error;
        }
      }
    }
    await this.initialize();
  }

  /**
   * Create the database and schema if missing. Must be called before any
   * other method to open the connection.
   */
  async initialize(): Promise<void> {
    await fs.mkdir(path.dirname(this.dbpath), { recursive: true });
    this.database = new Database(this.dbpath, { create: true });
    this.database.run('PRAGMA foreign_keys = ON;');
    this.database.run('PRAGMA journal_mode = WAL;');

    this.database.run(
      `CREATE TABLE IF NOT EXISTS person (
        id TEXT NOT NULL PRIMARY KEY,
        name TEXT,
        thumbnail_face TEXT,
        hidden INTEGER NOT NULL DEFAULT 0,
        created_at INTEGER NOT NULL
      ) STRICT`
    );

    // asset_id has no SQL foreign key: the asset lives in a different store.
    this.database.run(
      `CREATE TABLE IF NOT EXISTS face (
        id TEXT NOT NULL PRIMARY KEY,
        asset_id TEXT NOT NULL,
        person_id TEXT REFERENCES person(id) ON DELETE SET NULL,
        bbox TEXT NOT NULL,
        embedding BLOB NOT NULL,
        thumbnail BLOB NOT NULL,
        detector_score REAL,
        model_version TEXT NOT NULL
      ) STRICT`
    );

    this.database.run(
      `CREATE TABLE IF NOT EXISTS synthetic_jobs (
        id INTEGER PRIMARY KEY,
        asset_id TEXT NOT NULL,
        kind TEXT NOT NULL,
        priority INTEGER NOT NULL DEFAULT 0,
        attempts INTEGER NOT NULL DEFAULT 0,
        last_error TEXT,
        enqueued_at INTEGER NOT NULL,
        not_before INTEGER NOT NULL DEFAULT 0
      ) STRICT`
    );
    // Belt-and-braces for DBs that pre-date the not_before column. Ignore the
    // "duplicate column name" error from SQLite when the column is already
    // present (i.e. a fresh CREATE TABLE just installed it).
    try {
      this.database.run(
        'ALTER TABLE synthetic_jobs ADD COLUMN not_before INTEGER NOT NULL DEFAULT 0'
      );
    } catch (error: any) {
      if (!/duplicate column name/i.test(String(error?.message ?? error))) {
        throw error;
      }
    }

    this.database.run(
      'CREATE INDEX IF NOT EXISTS face_by_person ON face(person_id)'
    );
    this.database.run(
      'CREATE INDEX IF NOT EXISTS face_by_asset ON face(asset_id)'
    );
    this.database.run(
      'CREATE INDEX IF NOT EXISTS face_by_version ON face(model_version)'
    );
    this.database.run(
      `CREATE INDEX IF NOT EXISTS jobs_ready
         ON synthetic_jobs(not_before ASC, priority DESC, enqueued_at ASC)`
    );
  }

  /** @inheritDoc */
  async enqueueJob(
    assetId: string,
    kind: JobKind,
    priority = 0
  ): Promise<number> {
    const row = this.database!
      .query(
        `INSERT INTO synthetic_jobs
           (asset_id, kind, priority, attempts, last_error, enqueued_at)
         VALUES (?, ?, ?, 0, NULL, ?)
         RETURNING id`
      )
      .get(assetId, kind, priority, now()) as { id: number };
    return row.id;
  }

  /** @inheritDoc */
  async claimNextJob(): Promise<SyntheticJob | null> {
    // Atomic claim: delete the highest-priority, oldest, currently-eligible row
    // and return it in one statement so two workers can never claim the same
    // job. Jobs whose `not_before` is still in the future are skipped, which is
    // how retry backoff is enforced.
    const row = this.database!
      .query(
        `DELETE FROM synthetic_jobs
          WHERE id = (
            SELECT id FROM synthetic_jobs
             WHERE not_before <= ?
             ORDER BY priority DESC, enqueued_at ASC, id ASC
             LIMIT 1
          )
        RETURNING id, asset_id, kind, priority, attempts, last_error, enqueued_at`
      )
      .get(now()) as JobRow | undefined;
    return row ? jobFromRow(row) : null;
  }

  /** @inheritDoc */
  async requeueJob(
    job: SyntheticJob,
    error: string,
    delaySeconds = 0
  ): Promise<number> {
    const attempts = job.attempts + 1;
    const notBefore = now() + Math.max(0, Math.trunc(delaySeconds));
    // Re-enqueue at the same priority and original enqueue time so the job
    // keeps its place in line, but mark it not-claimable until `not_before`
    // elapses. The row is visible to `hasPendingJob` immediately, so backfill
    // / retry won't enqueue a duplicate during the backoff window.
    this.database!.query(
      `INSERT INTO synthetic_jobs
         (asset_id, kind, priority, attempts, last_error, enqueued_at, not_before)
       VALUES (?, ?, ?, ?, ?, ?, ?)`
    ).run(
      job.assetId,
      job.kind,
      job.priority,
      attempts,
      error,
      job.enqueuedAt,
      notBefore
    );
    return attempts;
  }

  /** @inheritDoc */
  async pendingJobCount(kind?: JobKind): Promise<number> {
    const row = kind
      ? (this.database!
          .query(
            'SELECT COUNT(*) AS count FROM synthetic_jobs WHERE kind = ?'
          )
          .get(kind) as { count: number })
      : (this.database!
          .query('SELECT COUNT(*) AS count FROM synthetic_jobs')
          .get() as { count: number });
    return row.count;
  }

  /** @inheritDoc */
  async hasPendingJob(assetId: string, kind: JobKind): Promise<boolean> {
    const row = this.database!
      .query(
        `SELECT 1 FROM synthetic_jobs WHERE asset_id = ? AND kind = ? LIMIT 1`
      )
      .get(assetId, kind);
    return row !== null && row !== undefined;
  }

  /** @inheritDoc */
  async fetchPeopleByAssetIds(
    _assetIds: string[]
  ): Promise<Map<string, Person[]>> {
    throw new Error('FaceStore.fetchPeopleByAssetIds is a Phase 2 feature');
  }

  /** @inheritDoc */
  async assetIdsByPerson(
    _personId: string,
    _offset: number,
    _limit: number
  ): Promise<{ ids: string[]; total: number }> {
    throw new Error('FaceStore.assetIdsByPerson is a Phase 2 feature');
  }

  /** @inheritDoc */
  async deleteByAssetId(_assetId: string): Promise<void> {
    throw new Error('FaceStore.deleteByAssetId is a Phase 2 feature');
  }
}

export { SqliteFaceStore };
