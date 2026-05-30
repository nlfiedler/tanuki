//
// Copyright (c) 2026 Nathan Fiedler
//
import { type SyntheticJob } from 'tanuki/server/domain/entities/face.ts';

/**
 * Performs the actual extraction for a claimed synthetic-data job: runs the
 * ML detector against the asset's bytes and persists the results, setting the
 * asset's `syntheticStatus` to READY on success.
 *
 * Implementations throw on a recoverable failure; the {@link
 * import('./synthetic-worker-pool.ts')} worker pool owns retry/backoff and
 * records the terminal FAILED state. The concrete detector-backed
 * implementation (local ONNX or Namazu HTTP) arrives in a later slice.
 */
interface SyntheticJobProcessor {
  /**
   * Process a single claimed job to completion. Resolve on success (results
   * persisted); throw to signal a retryable failure.
   *
   * @param job - the job claimed from the queue.
   */
  process(job: SyntheticJob): Promise<void>;
}

export { type SyntheticJobProcessor };
