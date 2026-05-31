//
// Copyright (c) 2026 Nathan Fiedler
//
import {
  type Face,
  type JobKind,
  type Person,
  type PersonSummary,
  type SyntheticJob
} from 'tanuki/server/domain/entities/face.ts';
import { type SyntheticStatus } from 'tanuki/server/domain/entities/synthetic-data.ts';

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

  // --- faces / people -------------------------------------------------------

  /**
   * Resolve the people appearing in each of the given assets. Batched for the
   * `synthetic.people` DataLoader. The returned map has an entry for every
   * requested asset id (an empty array when no faces are clustered on it).
   * People with a `null` cluster (unassigned faces) are not surfaced. Hidden
   * people are included — the caller decides whether to show them.
   *
   * @param assetIds - assets to resolve people for.
   */
  fetchPeopleByAssetIds(
    assetIds: string[]
  ): Promise<Map<string, PersonSummary[]>>;

  /**
   * Page the asset ids associated with a person, most-recently-detected first.
   * Ordering is by face insertion order (rowid) rather than asset date, which
   * the face store does not know; the use case can re-sort after materializing
   * the assets from the record store.
   *
   * @param personId - the person whose assets to page.
   * @param offset - zero-based page offset.
   * @param limit - maximum ids to return.
   * @returns the page of distinct asset ids plus the total count.
   */
  assetIdsByPerson(
    personId: string,
    offset: number,
    limit: number
  ): Promise<{ ids: string[]; total: number }>;

  /**
   * Remove all face rows for an asset, then cascade-delete any person rows
   * left with no faces. Called by the asset-deletion use case to maintain
   * cross-store consistency in place of an `ON DELETE CASCADE`.
   *
   * @param assetId - asset whose faces should be removed.
   */
  deleteByAssetId(assetId: string): Promise<void>;

  // --- clustering / ingestion ----------------------------------------------

  /**
   * Persist a detected face. The `personId` should already be assigned (by the
   * clustering step) or left null for an unclustered face.
   *
   * @param face - the face row to insert.
   */
  insertFace(face: Face): Promise<void>;

  /**
   * Find the most similar already-clustered face within the same
   * `modelVersion` and return its owning person together with the cosine
   * similarity. Embeddings are L2-normalized, so cosine similarity is the dot
   * product. Returns null when no clustered face exists for that version.
   *
   * The caller compares `score` against the clustering threshold to decide
   * whether to join the matched person or start a new cluster.
   *
   * @param embedding - the query embedding (512 floats, L2-normalized).
   * @param modelVersion - only faces from this model version are compared.
   */
  nearestPerson(
    embedding: Float32Array,
    modelVersion: string
  ): Promise<{ personId: string; score: number } | null>;

  /**
   * Create a new, unnamed person row and return it.
   */
  createPerson(): Promise<Person>;

  // --- lifecycle / queries --------------------------------------------------

  /**
   * List people for the People page, each enriched with face count and
   * representative face. Sorted by creation order.
   *
   * @param includeHidden - when false, omit people flagged hidden.
   */
  listPeople(includeHidden: boolean): Promise<PersonSummary[]>;

  /**
   * Fetch a single person enriched with face count and representative face, or
   * null if no such person exists.
   *
   * @param id - the person id.
   */
  getPersonSummary(id: string): Promise<PersonSummary | null>;

  /**
   * Return the ids of every person whose user-assigned name matches (exact,
   * case-insensitive). Backs name-based `person:` searches. Returns an empty
   * array when nothing matches; unnamed people never match.
   *
   * @param name - the name to match.
   */
  personIdsByName(name: string): Promise<string[]>;

  /**
   * Return every face clustered under a person, for the cluster-detail view.
   * The heavy `embedding` and `thumbnail` blobs are included so the caller can
   * render crops without a second round-trip.
   *
   * @param personId - the person whose faces to return.
   */
  facesForPerson(personId: string): Promise<Face[]>;

  /**
   * Return the stored ~128px JPEG crop for a face, or null if no such face
   * exists. Backs the `/faces/:id/thumb` route.
   *
   * @param faceId - the face id.
   */
  faceThumbnail(faceId: string): Promise<Uint8Array | null>;

  /**
   * Set (or clear, with null) a person's user-assigned name.
   *
   * @param id - the person id.
   * @param name - the new name, or null to clear.
   */
  renamePerson(id: string, name: string | null): Promise<void>;

  /**
   * Merge the source person into the target: reassign all of the source's
   * faces to the target, then delete the now-empty source person row.
   *
   * @param sourceId - person to merge from (deleted afterwards).
   * @param targetId - person to merge into (survives).
   */
  mergePeople(sourceId: string, targetId: string): Promise<void>;

  /**
   * Reassign the given faces to a person. When `personId` is null a new person
   * is created to receive them (the split case). Any source person left with
   * no faces afterwards is cascade-deleted. Returns the id of the destination
   * person.
   *
   * @param faceIds - faces to move.
   * @param personId - destination person, or null to create one.
   * @returns the destination person id.
   */
  reassignFaces(
    faceIds: string[],
    personId: string | null
  ): Promise<string>;

  /**
   * Flip a person's hidden flag. Hidden people are excluded from the People
   * page but their face rows are preserved.
   *
   * @param id - the person id.
   * @param hidden - the new flag value.
   */
  hidePerson(id: string, hidden: boolean): Promise<void>;

  /**
   * Pin a specific face as a person's representative thumbnail. The face must
   * belong to the person.
   *
   * @param id - the person id.
   * @param faceId - the face to use as the thumbnail.
   */
  setPersonThumbnail(id: string, faceId: string): Promise<void>;

  // --- faces extraction status ---------------------------------------------
  //
  // Faces status lives here rather than on the asset record so the labels path
  // (and the three record backends) stay untouched. The GraphQL
  // `syntheticStatus` field is the worse of the record's labels status and
  // this faces status. Absence of a row means PENDING.

  /**
   * Record the terminal faces-extraction status for an asset. Passing
   * `PENDING` clears any stored status (back to the implicit default), which
   * is how backfill / retry reset an asset before re-enqueuing.
   *
   * @param assetId - the asset whose faces status to record.
   * @param status - READY / FAILED, or PENDING to clear.
   */
  setFacesStatus(assetId: string, status: SyntheticStatus): Promise<void>;

  /**
   * Fetch the faces-extraction status for the given assets in one batched
   * call. Every requested id appears in the map; assets with no stored status
   * default to PENDING.
   *
   * @param assetIds - assets to fetch faces status for.
   */
  fetchFacesStatus(assetIds: string[]): Promise<Map<string, SyntheticStatus>>;

  /**
   * Return every asset id currently recorded at the given faces status. Used
   * by `retrySyntheticJobs` to find FAILED faces work.
   *
   * @param status - the status to match (typically FAILED).
   */
  assetIdsWithFacesStatus(status: SyntheticStatus): Promise<string[]>;

  /**
   * Count the assets currently recorded at the given faces status. Backs the
   * `syntheticJobStatus` query's progress readout (READY / FAILED tallies)
   * without materializing the id list. Assets with no stored status (implicit
   * PENDING) are not counted.
   *
   * @param status - the status to match.
   */
  facesStatusCount(status: SyntheticStatus): Promise<number>;

  /**
   * Return, for each requested asset, the set of `model_version` values across
   * its face rows. Used by `backfillFaceRecognition` to detect assets whose
   * faces were produced by a stale model and need reprocessing. Assets with no
   * faces are absent from the map.
   *
   * @param assetIds - assets to inspect.
   */
  modelVersionsByAssets(
    assetIds: string[]
  ): Promise<Map<string, Set<string>>>;

  // --- cross-store consistency ----------------------------------------------

  /**
   * Return every distinct `asset_id` that has at least one face row. Used by
   * the defensive orphan sweep to find faces whose asset no longer exists.
   */
  allFaceAssetIds(): Promise<string[]>;
}

export { type FaceStore };
