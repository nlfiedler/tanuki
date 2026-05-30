//
// Copyright (c) 2026 Nathan Fiedler
//
import {
  type JobKind,
  type Person,
  type SyntheticJob
} from 'tanuki/server/domain/entities/face.ts';

/**
 * Dedicated SQLite store for face/person data and the background-extraction
 * job queue. A single implementation backs every deployment regardless of
 * which `RecordRepository` holds the assets, because binary embeddings, face
 * crops, high-churn cluster mutations, and the job queue all want
 * relational/BLOB semantics that CouchDB/PouchDB serve poorly.
 *
 * Slice 2 implements the `synthetic_jobs` queue; the face/person methods are
 * declared here but not yet implemented (Phase 2).
 */
interface FaceStore {
  // --- synthetic_jobs queue -------------------------------------------------

  /**
   * Append a job to the queue.
   *
   * @param assetId - asset the job operates on.
   * @param kind - which extraction pipeline to run.
   * @param priority - higher runs first; live imports 10, backfill 0.
   * @returns the new job's row id.
   */
  enqueueJob(assetId: string, kind: JobKind, priority?: number): Promise<number>;

  /**
   * Atomically remove and return the next job to run — highest priority,
   * then oldest enqueued. The row is deleted on claim; the caller holds it in
   * memory and re-enqueues it (via {@link failJob}) only if processing fails.
   *
   * @returns the claimed job, or null if the queue is empty.
   */
  claimNextJob(): Promise<SyntheticJob | null>;

  /**
   * Put a previously claimed job back on the queue for another attempt, with
   * `attempts` incremented and `lastError` set. The job keeps its priority and
   * original enqueue time so it retains its place in line, but `claimNextJob`
   * will skip it until `delaySeconds` have elapsed — that is how the worker
   * pool's exponential backoff is enforced (while still keeping the row
   * visible to {@link hasPendingJob} so backfill won't duplicate).
   *
   * @param job - the job returned by {@link claimNextJob}.
   * @param error - human-readable failure message to persist.
   * @param delaySeconds - seconds from now before the job is eligible again.
   * @returns the job's new attempt count.
   */
  requeueJob(
    job: SyntheticJob,
    error: string,
    delaySeconds?: number
  ): Promise<number>;

  /**
   * Count jobs currently waiting in the queue, optionally filtered by kind.
   *
   * @param kind - restrict to one pipeline, or omit for all.
   */
  pendingJobCount(kind?: JobKind): Promise<number>;

  /**
   * Whether a job of the given kind is already queued for an asset. Used by
   * backfill to stay idempotent — it skips assets that already have work
   * pending. Note this cannot see a job currently being processed (claim
   * removes it from the queue), which leaves a small, tolerable race window.
   *
   * @param assetId - asset to check.
   * @param kind - pipeline to check for.
   */
  hasPendingJob(assetId: string, kind: JobKind): Promise<boolean>;

  // --- faces / people (Phase 2) --------------------------------------------

  /**
   * Resolve the people appearing in each of the given assets. Batched for the
   * `synthetic.people` DataLoader.
   *
   * Not implemented in Slice 2.
   */
  fetchPeopleByAssetIds(assetIds: string[]): Promise<Map<string, Person[]>>;

  /**
   * Page the asset ids associated with a person, newest first.
   *
   * Not implemented in Slice 2.
   */
  assetIdsByPerson(
    personId: string,
    offset: number,
    limit: number
  ): Promise<{ ids: string[]; total: number }>;

  /**
   * Remove all face rows for an asset (and clean up any now-empty person
   * rows). Called by the asset-deletion use case to maintain cross-store
   * consistency in place of an `ON DELETE CASCADE`.
   *
   * Not implemented in Slice 2.
   */
  deleteByAssetId(assetId: string): Promise<void>;
}

export { type FaceStore };
