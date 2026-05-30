//
// Copyright (c) 2026 Nathan Fiedler
//
import { type Asset } from 'tanuki/server/domain/entities/asset.ts';
import { type DetectedFace } from 'tanuki/server/domain/entities/face.ts';

/**
 * Produces synthetic data from an asset's pixels: label classification
 * (Phase 1) and face detection + embedding (Phase 2). Implementations are
 * backend-symmetric: a local in-process ONNX runtime, or (later) an HTTP
 * push-down to a Namazu blob store. Both emit the same curated display labels
 * and byte-comparable embeddings for the same image.
 */
interface SyntheticDetector {
  /**
   * Classify an image asset and return curated display labels, highest
   * confidence first (already de-duplicated and capped). Non-image assets and
   * images that yield nothing above the score floor return an empty array.
   *
   * @param asset - the asset to classify.
   */
  detectLabels(asset: Asset): Promise<string[]>;

  /**
   * Detect every face in an image asset and return each one's bounding box,
   * L2-normalized embedding, aligned crop thumbnail, detector score, and model
   * version. Non-image assets, and images with no face above the detection
   * threshold, return an empty array.
   *
   * @param asset - the asset to scan for faces.
   */
  detectFaces(asset: Asset): Promise<DetectedFace[]>;

  /**
   * The embedding model version this detector currently produces (e.g.
   * `mobilefacenet-v1`). Backfill compares stored faces against this to find
   * assets whose faces are stale and need reprocessing.
   */
  faceModelVersion(): string;
}

export { type SyntheticDetector };
