//
// Copyright (c) 2026 Nathan Fiedler
//
import { type Asset } from 'tanuki/server/domain/entities/asset.ts';

/**
 * Produces synthetic data from an asset's pixels. Phase 1 covers label
 * classification. Implementations are backend-symmetric: a local in-process
 * ONNX runtime, or (later) an HTTP push-down to a Namazu blob store. Both emit
 * the same curated display labels for the same image.
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
}

export { type SyntheticDetector };
