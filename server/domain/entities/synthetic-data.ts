//
// Copyright (c) 2026 Nathan Fiedler
//

/**
 * Progress of synthetic-data extraction for an asset. Every asset has a
 * status; `PENDING` is the default until a worker (or backfill) records a
 * terminal value.
 */
enum SyntheticStatus {
  /** No worker has finished processing the asset yet. */
  PENDING = 'PENDING',
  /** Synthetic data has been extracted and stored. */
  READY = 'READY',
  /** All retry attempts failed; operator action required. */
  FAILED = 'FAILED'
}

/**
 * Synthetic data derived from an image asset by machine-learning models.
 * Phase 1 covers the label-tagging fields; the `people` collection arrives
 * with Phase 2 (face recognition) and is resolved separately from the face
 * store at query time, so this entity does not carry it.
 */
class SyntheticData {
  /** Curated display labels (post-curation), ordered by descending score. */
  labels: string[] = [];
  /**
   * Top label, denormalized for fast queries on `assets-by-primary-label`.
   * `null` if the classifier returned nothing above the score floor.
   */
  primaryLabel: string | null = null;

  /** Returns true if any label or a primary label is present. */
  hasValues(): boolean {
    return this.labels.length > 0 || this.primaryLabel !== null;
  }
}

/**
 * A row in the Labels page index: one distinct `primaryLabel` value with the
 * number of assets carrying it and the id of a representative asset (the
 * most-recently-dated one) for the page's thumbnail.
 */
class LabelEntry {
  label: string;
  count: number;
  thumbnailAssetId: string;

  constructor(label: string, count: number, thumbnailAssetId: string) {
    this.label = label;
    this.count = count;
    this.thumbnailAssetId = thumbnailAssetId;
  }
}

export { LabelEntry, SyntheticData, SyntheticStatus };
