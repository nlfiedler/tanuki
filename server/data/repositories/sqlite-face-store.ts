//
// Copyright (c) 2026 Nathan Fiedler
//
import assert from 'node:assert';
import crypto from 'node:crypto';
import fs from 'node:fs/promises';
import path from 'node:path';
import { Database } from 'bun:sqlite';
import {
  Face,
  type JobKind,
  Person,
  type PersonSummary,
  SyntheticJob
} from 'tanuki/server/domain/entities/face.ts';
import { SyntheticStatus } from 'tanuki/server/domain/entities/synthetic-data.ts';
import { type FaceStore } from 'tanuki/server/domain/repositories/face-store.ts';
import { type SettingsRepository } from 'tanuki/server/domain/repositories/settings-repository.ts';
import { dot } from 'tanuki/server/data/synthetic/face-align.ts';

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

interface PersonRow {
  id: string;
  name: string | null;
  thumbnail_face: string | null;
  hidden: number;
  created_at: number;
}

function personFromRow(row: PersonRow): Person {
  return new Person(
    row.id,
    row.name,
    row.thumbnail_face,
    row.hidden !== 0,
    row.created_at
  );
}

interface FaceRow {
  id: string;
  asset_id: string;
  person_id: string | null;
  bbox: string;
  embedding: Uint8Array;
  thumbnail: Uint8Array;
  detector_score: number | null;
  model_version: string;
}

/**
 * Decode a stored embedding BLOB into a Float32Array. The bytes come back from
 * SQLite as a Uint8Array whose backing buffer may not be 4-byte aligned, so we
 * copy into a fresh, aligned ArrayBuffer before viewing it as floats.
 */
function embeddingFromBlob(blob: Uint8Array): Float32Array {
  const copy = blob.buffer.slice(
    blob.byteOffset,
    blob.byteOffset + blob.byteLength
  );
  return new Float32Array(copy);
}

/** Encode an embedding as a BLOB-ready byte view (no copy). */
function embeddingToBlob(embedding: Float32Array): Uint8Array {
  return new Uint8Array(
    embedding.buffer,
    embedding.byteOffset,
    embedding.byteLength
  );
}

function faceFromRow(row: FaceRow): Face {
  return new Face(
    row.id,
    row.asset_id,
    JSON.parse(row.bbox) as [number, number, number, number],
    embeddingFromBlob(row.embedding),
    row.thumbnail,
    row.model_version,
    row.person_id,
    row.detector_score
  );
}

/**
 * SQL fragment ordering a person's faces so the "best" representative comes
 * first: largest bounding-box area, breaking ties by detector score. Used both
 * to pick a default thumbnail and (implicitly) to keep that choice stable.
 */
const REPRESENTATIVE_ORDER =
  `ORDER BY (CAST(json_extract(bbox, '$[2]') AS REAL) * ` +
  `CAST(json_extract(bbox, '$[3]') AS REAL)) DESC, ` +
  `detector_score DESC`;

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

    // Terminal faces-extraction status per asset (absence = PENDING). Kept in
    // the face store so the labels path and the three record backends are
    // untouched; the GraphQL status is the worse of this and the labels status.
    this.database.run(
      `CREATE TABLE IF NOT EXISTS face_status (
        asset_id TEXT NOT NULL PRIMARY KEY,
        status TEXT NOT NULL,
        updated_at INTEGER NOT NULL
      ) STRICT`
    );

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
    assetIds: string[]
  ): Promise<Map<string, PersonSummary[]>> {
    const result = new Map<string, PersonSummary[]>();
    for (const id of assetIds) result.set(id, []);
    if (assetIds.length === 0) return result;

    const placeholders = assetIds.map(() => '?').join(',');
    const rows = this.database!
      .query(
        `SELECT DISTINCT asset_id, person_id FROM face
          WHERE person_id IS NOT NULL AND asset_id IN (${placeholders})`
      )
      .all(...assetIds) as { asset_id: string; person_id: string }[];

    // Load the distinct people in one query and summarize them in a batch,
    // then fan each summary out to every asset it appears in.
    const personIds = [...new Set(rows.map((row) => row.person_id))];
    const personPlaceholders = personIds.map(() => '?').join(',');
    const personRows = this.database!
      .query(
        `SELECT id, name, thumbnail_face, hidden, created_at FROM person
          WHERE id IN (${personPlaceholders})`
      )
      .all(...personIds) as PersonRow[];
    const summaries = new Map<string, PersonSummary>(
      this.summarizePersonRows(personRows).map((s) => [s.person.id, s])
    );
    for (const row of rows) {
      const summary = summaries.get(row.person_id);
      if (summary) result.get(row.asset_id)!.push(summary);
    }
    return result;
  }

  /** @inheritDoc */
  async assetIdsByPerson(
    personId: string,
    offset: number,
    limit: number
  ): Promise<{ ids: string[]; total: number }> {
    const total = (
      this.database!
        .query(
          'SELECT COUNT(DISTINCT asset_id) AS count FROM face WHERE person_id = ?'
        )
        .get(personId) as { count: number }
    ).count;
    // Group by asset so an asset with several faces of the same person appears
    // once; order by the newest contributing face (highest rowid).
    const rows = this.database!
      .query(
        `SELECT asset_id, MAX(rowid) AS r FROM face
          WHERE person_id = ?
          GROUP BY asset_id
          ORDER BY r DESC
          LIMIT ? OFFSET ?`
      )
      .all(personId, limit, offset) as { asset_id: string }[];
    return { ids: rows.map((row) => row.asset_id), total };
  }

  /** @inheritDoc */
  async deleteByAssetId(assetId: string): Promise<void> {
    this.database!.transaction(() => {
      const affected = this.database!
        .query(
          `SELECT DISTINCT person_id FROM face
            WHERE asset_id = ? AND person_id IS NOT NULL`
        )
        .all(assetId) as { person_id: string }[];
      this.database!.query('DELETE FROM face WHERE asset_id = ?').run(assetId);
      this.database!
        .query('DELETE FROM face_status WHERE asset_id = ?')
        .run(assetId);
      this.cleanupEmptyPeople(affected.map((row) => row.person_id));
    })();
  }

  /** @inheritDoc */
  async setFacesStatus(
    assetId: string,
    status: SyntheticStatus
  ): Promise<void> {
    if (status === SyntheticStatus.PENDING) {
      // PENDING is the implicit default; clear any stored row.
      this.database!
        .query('DELETE FROM face_status WHERE asset_id = ?')
        .run(assetId);
      return;
    }
    this.database!.query(
      `INSERT INTO face_status (asset_id, status, updated_at)
       VALUES (?, ?, ?)
       ON CONFLICT(asset_id) DO UPDATE SET status = excluded.status,
         updated_at = excluded.updated_at`
    ).run(assetId, status, now());
  }

  /** @inheritDoc */
  async fetchFacesStatus(
    assetIds: string[]
  ): Promise<Map<string, SyntheticStatus>> {
    const result = new Map<string, SyntheticStatus>();
    for (const id of assetIds) result.set(id, SyntheticStatus.PENDING);
    if (assetIds.length === 0) return result;
    const placeholders = assetIds.map(() => '?').join(',');
    const rows = this.database!
      .query(
        `SELECT asset_id, status FROM face_status
          WHERE asset_id IN (${placeholders})`
      )
      .all(...assetIds) as { asset_id: string; status: string }[];
    for (const row of rows) {
      result.set(row.asset_id, parseFacesStatus(row.status));
    }
    return result;
  }

  /** @inheritDoc */
  async assetIdsWithFacesStatus(
    status: SyntheticStatus
  ): Promise<string[]> {
    const rows = this.database!
      .query('SELECT asset_id FROM face_status WHERE status = ?')
      .all(status) as { asset_id: string }[];
    return rows.map((row) => row.asset_id);
  }

  /** @inheritDoc */
  async facesStatusCount(status: SyntheticStatus): Promise<number> {
    const row = this.database!
      .query('SELECT COUNT(*) AS count FROM face_status WHERE status = ?')
      .get(status) as { count: number };
    return row.count;
  }

  /** @inheritDoc */
  async modelVersionsByAssets(
    assetIds: string[]
  ): Promise<Map<string, Set<string>>> {
    const result = new Map<string, Set<string>>();
    if (assetIds.length === 0) return result;
    const placeholders = assetIds.map(() => '?').join(',');
    const rows = this.database!
      .query(
        `SELECT DISTINCT asset_id, model_version FROM face
          WHERE asset_id IN (${placeholders})`
      )
      .all(...assetIds) as { asset_id: string; model_version: string }[];
    for (const row of rows) {
      let versions = result.get(row.asset_id);
      if (!versions) {
        versions = new Set<string>();
        result.set(row.asset_id, versions);
      }
      versions.add(row.model_version);
    }
    return result;
  }

  /** @inheritDoc */
  async insertFace(face: Face): Promise<void> {
    this.database!.query(
      `INSERT INTO face
         (id, asset_id, person_id, bbox, embedding, thumbnail,
          detector_score, model_version)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?)`
    ).run(
      face.id,
      face.assetId,
      face.personId,
      JSON.stringify(face.bbox),
      embeddingToBlob(face.embedding),
      face.thumbnail,
      face.detectorScore,
      face.modelVersion
    );
  }

  /** @inheritDoc */
  async nearestPerson(
    embedding: Float32Array,
    modelVersion: string
  ): Promise<{ personId: string; score: number } | null> {
    const rows = this.database!
      .query(
        `SELECT person_id, embedding FROM face
          WHERE person_id IS NOT NULL AND model_version = ?`
      )
      .all(modelVersion) as { person_id: string; embedding: Uint8Array }[];
    let bestPerson: string | null = null;
    let bestScore = -Infinity;
    for (const row of rows) {
      const other = embeddingFromBlob(row.embedding);
      const score = dot(embedding, other);
      if (score > bestScore) {
        bestScore = score;
        bestPerson = row.person_id;
      }
    }
    return bestPerson === null
      ? null
      : { personId: bestPerson, score: bestScore };
  }

  /** @inheritDoc */
  async createPerson(): Promise<Person> {
    const id = crypto.randomUUID();
    const createdAt = now();
    this.insertPersonRow(id, createdAt);
    return new Person(id, null, null, false, createdAt);
  }

  /** Insert a bare, unnamed person row. Synchronous for use inside transactions. */
  private insertPersonRow(id: string, createdAt: number): void {
    this.database!.query(
      `INSERT INTO person (id, name, thumbnail_face, hidden, created_at)
       VALUES (?, NULL, NULL, 0, ?)`
    ).run(id, createdAt);
  }

  /** @inheritDoc */
  async listPeople(includeHidden: boolean): Promise<PersonSummary[]> {
    const rows = this.database!
      .query(
        `SELECT id, name, thumbnail_face, hidden, created_at FROM person
          ${includeHidden ? '' : 'WHERE hidden = 0'}
          ORDER BY created_at ASC, id ASC`
      )
      .all() as PersonRow[];
    return this.summarizePersonRows(rows);
  }

  /** @inheritDoc */
  async getPersonSummary(id: string): Promise<PersonSummary | null> {
    return this.summarizePersonById(id);
  }

  /** @inheritDoc */
  async personIdsByName(name: string): Promise<string[]> {
    const rows = this.database!
      .query(
        `SELECT id FROM person
          WHERE name IS NOT NULL AND LOWER(name) = LOWER(?)`
      )
      .all(name) as { id: string }[];
    return rows.map((row) => row.id);
  }

  /** @inheritDoc */
  async facesForPerson(personId: string): Promise<Face[]> {
    const rows = this.database!
      .query(
        `SELECT id, asset_id, person_id, bbox, embedding, thumbnail,
                detector_score, model_version
           FROM face WHERE person_id = ? ORDER BY rowid ASC`
      )
      .all(personId) as FaceRow[];
    return rows.map((row) => faceFromRow(row));
  }

  /** @inheritDoc */
  async faceThumbnail(faceId: string): Promise<Uint8Array | null> {
    const row = this.database!
      .query('SELECT thumbnail FROM face WHERE id = ?')
      .get(faceId) as { thumbnail: Uint8Array } | undefined;
    return row ? row.thumbnail : null;
  }

  /** @inheritDoc */
  async renamePerson(id: string, name: string | null): Promise<void> {
    // Normalize blank/whitespace-only names to null so "unnamed" is a single
    // canonical state rather than a mix of null and "".
    const normalized = name && name.trim().length > 0 ? name.trim() : null;
    this.database!.query('UPDATE person SET name = ? WHERE id = ?').run(
      normalized,
      id
    );
  }

  /** @inheritDoc */
  async mergePeople(sourceId: string, targetId: string): Promise<void> {
    if (sourceId === targetId) return;
    this.database!.transaction(() => {
      this.database!.query(
        'UPDATE face SET person_id = ? WHERE person_id = ?'
      ).run(targetId, sourceId);
      this.database!.query('DELETE FROM person WHERE id = ?').run(sourceId);
    })();
  }

  /** @inheritDoc */
  async reassignFaces(
    faceIds: string[],
    personId: string | null
  ): Promise<string> {
    if (faceIds.length === 0) {
      throw new Error('reassignFaces requires at least one face');
    }
    return this.database!.transaction(() => {
      const placeholders = faceIds.map(() => '?').join(',');
      // Remember the source clusters so we can prune any that empty out.
      const sources = this.database!
        .query(
          `SELECT DISTINCT person_id FROM face
            WHERE person_id IS NOT NULL AND id IN (${placeholders})`
        )
        .all(...faceIds) as { person_id: string }[];

      let destination = personId;
      if (destination === null) {
        destination = crypto.randomUUID();
        this.insertPersonRow(destination, now());
      }
      this.database!
        .query(
          `UPDATE face SET person_id = ? WHERE id IN (${placeholders})`
        )
        .run(destination, ...faceIds);

      this.cleanupEmptyPeople(
        sources.map((row) => row.person_id).filter((id) => id !== destination)
      );
      return destination;
    })();
  }

  /** @inheritDoc */
  async hidePerson(id: string, hidden: boolean): Promise<void> {
    this.database!.query('UPDATE person SET hidden = ? WHERE id = ?').run(
      hidden ? 1 : 0,
      id
    );
  }

  /** @inheritDoc */
  async setPersonThumbnail(id: string, faceId: string): Promise<void> {
    const belongs = this.database!
      .query('SELECT 1 FROM face WHERE id = ? AND person_id = ?')
      .get(faceId, id);
    if (!belongs) {
      throw new Error(`face ${faceId} does not belong to person ${id}`);
    }
    this.database!.query(
      'UPDATE person SET thumbnail_face = ? WHERE id = ?'
    ).run(faceId, id);
  }

  /** @inheritDoc */
  async allFaceAssetIds(): Promise<string[]> {
    const rows = this.database!
      .query('SELECT DISTINCT asset_id FROM face')
      .all() as { asset_id: string }[];
    return rows.map((row) => row.asset_id);
  }

  /** Build a {@link PersonSummary} for a person id, or null if absent. */
  private summarizePersonById(id: string): PersonSummary | null {
    const row = this.database!
      .query(
        'SELECT id, name, thumbnail_face, hidden, created_at FROM person WHERE id = ?'
      )
      .get(id) as PersonRow | undefined;
    return row ? (this.summarizePersonRows([row])[0] ?? null) : null;
  }

  /**
   * Enrich many person rows with face count and resolved representative face
   * using three batched queries (counts grouped by person, the best face per
   * person via a window function, and pinned-thumbnail validity) instead of a
   * per-person fan-out — so list/loader paths issue O(1) queries regardless of
   * how many people are on the page. The representative is the explicit pinned
   * thumbnail while it still belongs to the person, otherwise the largest face
   * by bbox area (ties broken by detector score).
   */
  private summarizePersonRows(rows: PersonRow[]): PersonSummary[] {
    if (rows.length === 0) return [];
    const ids = rows.map((row) => row.id);
    const placeholders = ids.map(() => '?').join(',');

    const counts = new Map<string, number>();
    for (const row of this.database!
      .query(
        `SELECT person_id, COUNT(*) AS count FROM face
          WHERE person_id IN (${placeholders}) GROUP BY person_id`
      )
      .all(...ids) as { person_id: string; count: number }[]) {
      counts.set(row.person_id, row.count);
    }

    // One representative face per person: rank within each person and keep #1.
    const best = new Map<string, string>();
    for (const row of this.database!
      .query(
        `SELECT person_id, id FROM (
           SELECT person_id, id,
             ROW_NUMBER() OVER (PARTITION BY person_id ${REPRESENTATIVE_ORDER})
               AS rn
           FROM face WHERE person_id IN (${placeholders})
         ) WHERE rn = 1`
      )
      .all(...ids) as { person_id: string; id: string }[]) {
      best.set(row.person_id, row.id);
    }

    // A pinned thumbnail counts only while that face still belongs to the
    // person; validate all pinned faces in one query (keyed person:face).
    const pinned = rows
      .map((row) => row.thumbnail_face)
      .filter((face): face is string => face !== null);
    const validPinned = new Set<string>();
    if (pinned.length > 0) {
      const pinnedPlaceholders = pinned.map(() => '?').join(',');
      for (const row of this.database!
        .query(
          `SELECT id, person_id FROM face WHERE id IN (${pinnedPlaceholders})`
        )
        .all(...pinned) as { id: string; person_id: string | null }[]) {
        validPinned.add(`${row.person_id}:${row.id}`);
      }
    }

    return rows.map((row) => {
      const pinnedOk =
        row.thumbnail_face !== null &&
        validPinned.has(`${row.id}:${row.thumbnail_face}`);
      const representativeFaceId = pinnedOk
        ? row.thumbnail_face
        : (best.get(row.id) ?? null);
      return {
        person: personFromRow(row),
        faceCount: counts.get(row.id) ?? 0,
        representativeFaceId
      };
    });
  }

  /**
   * Delete any of the given person rows that no longer have faces. The last
   * face referencing a person being removed cascades to deleting the person
   * row (and its assigned name) per the cluster-lifecycle rules.
   */
  private cleanupEmptyPeople(personIds: string[]): void {
    for (const id of new Set(personIds)) {
      const remaining = this.database!
        .query('SELECT 1 FROM face WHERE person_id = ? LIMIT 1')
        .get(id);
      if (!remaining) {
        this.database!.query('DELETE FROM person WHERE id = ?').run(id);
      }
    }
  }
}

/** Parse a stored faces-status string, defaulting to PENDING on anything odd. */
function parseFacesStatus(value: string): SyntheticStatus {
  if (value === SyntheticStatus.READY) return SyntheticStatus.READY;
  if (value === SyntheticStatus.FAILED) return SyntheticStatus.FAILED;
  return SyntheticStatus.PENDING;
}

export { SqliteFaceStore };
