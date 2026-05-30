//
// Copyright (c) 2026 Nathan Fiedler
//

/**
 * Which extraction pipeline a `synthetic_jobs` row drives. Labels run the
 * image classifier; faces run detection + embedding. Both share one queue.
 */
type JobKind = 'labels' | 'faces';

/**
 * A unit of background synthetic-data extraction. Rows live in the face
 * store's `synthetic_jobs` queue; workers claim the highest-priority,
 * oldest-enqueued job and either complete it or re-enqueue it on a retryable
 * failure (with `attempts` incremented).
 */
class SyntheticJob {
  /** Auto-increment row id; assigned by the store on enqueue. */
  id: number;
  /** Asset the job operates on. No FK — the asset lives in another store. */
  assetId: string;
  /** Pipeline to run. */
  kind: JobKind;
  /** Higher runs first; live imports use 10, backfill uses 0. */
  priority: number;
  /** Number of times this job has already failed and been re-enqueued. */
  attempts: number;
  /** Message from the most recent failure, or null if never failed. */
  lastError: string | null;
  /** Epoch seconds when the (current incarnation of the) job was enqueued. */
  enqueuedAt: number;

  constructor(
    id: number,
    assetId: string,
    kind: JobKind,
    priority: number,
    attempts: number,
    lastError: string | null,
    enqueuedAt: number
  ) {
    this.id = id;
    this.assetId = assetId;
    this.kind = kind;
    this.priority = priority;
    this.attempts = attempts;
    this.lastError = lastError;
    this.enqueuedAt = enqueuedAt;
  }
}

/**
 * A clustered identity. People are opaque: the id is a UUID and the
 * (optional) name is user-assigned. Assets reference people transitively
 * through `Face.personId`, never directly. Phase 2 populates and mutates
 * these rows.
 */
class Person {
  /** Opaque UUID. */
  id: string;
  /** User-assigned display name, or null if unnamed. */
  name: string | null;
  /** `Face.id` chosen as the representative thumbnail, or null. */
  thumbnailFace: string | null;
  /** True when excluded from the People page. */
  hidden: boolean;
  /** Epoch seconds when the person row was created. */
  createdAt: number;

  constructor(
    id: string,
    name: string | null = null,
    thumbnailFace: string | null = null,
    hidden = false,
    createdAt = 0
  ) {
    this.id = id;
    this.name = name;
    this.thumbnailFace = thumbnailFace;
    this.hidden = hidden;
    this.createdAt = createdAt;
  }
}

/**
 * A single detected face: its bounding box, embedding vector, and a cropped
 * thumbnail, plus the model version that produced the embedding. Embeddings
 * are only ever compared within the same `modelVersion`. Phase 2 populates
 * these rows.
 */
class Face {
  /** Opaque id. */
  id: string;
  /** Asset the face was detected in (no SQL FK; cross-store reference). */
  assetId: string;
  /** Owning person cluster, or null if unassigned. */
  personId: string | null;
  /** Bounding box `[x, y, w, h]` in displayed-orientation pixels. */
  bbox: [number, number, number, number];
  /** L2-normalized embedding (512 floats for MobileFaceNet). */
  embedding: Float32Array;
  /** ~128px JPEG of the aligned face crop. */
  thumbnail: Uint8Array;
  /** Detector confidence, or null. */
  detectorScore: number | null;
  /** Model that produced the embedding, e.g. `mobilefacenet-v1`. */
  modelVersion: string;

  constructor(
    id: string,
    assetId: string,
    bbox: [number, number, number, number],
    embedding: Float32Array,
    thumbnail: Uint8Array,
    modelVersion: string,
    personId: string | null = null,
    detectorScore: number | null = null
  ) {
    this.id = id;
    this.assetId = assetId;
    this.bbox = bbox;
    this.embedding = embedding;
    this.thumbnail = thumbnail;
    this.modelVersion = modelVersion;
    this.personId = personId;
    this.detectorScore = detectorScore;
  }
}

export { Face, Person, SyntheticJob };
export type { JobKind };
