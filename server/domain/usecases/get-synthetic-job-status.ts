//
// Copyright (c) 2026 Nathan Fiedler
//
import assert from 'node:assert';
import { SyntheticStatus } from 'tanuki/server/domain/entities/synthetic-data.ts';
import { type FaceStore } from 'tanuki/server/domain/repositories/face-store.ts';

/** Snapshot of the synthetic-data extraction queue, for progress monitoring. */
interface SyntheticJobStatus {
  /** Jobs still waiting in the queue across both kinds (includes retry backoff). */
  queued: number;
  /** `faces` jobs still waiting in the queue. */
  facesQueued: number;
  /** Assets whose faces extraction completed successfully. */
  facesReady: number;
  /** Assets whose faces extraction failed after exhausting all retries. */
  facesFailed: number;
}

export default ({ faceStore }: { faceStore: FaceStore }) => {
  assert.ok(faceStore, 'face store must be defined');
  /**
   * Report the current state of the synthetic-data job queue so an operator can
   * watch a backfill drain to completion. `facesQueued` falling to zero (and
   * staying there) means the faces work is done; `facesReady` / `facesFailed`
   * give the success / give-up tallies. In-flight jobs are momentarily absent
   * from the queue while a worker holds them, so `facesQueued` can briefly read
   * a touch low — treat a sustained zero as done.
   */
  return async (): Promise<SyntheticJobStatus> => {
    return {
      queued: await faceStore.pendingJobCount(),
      facesQueued: await faceStore.pendingJobCount('faces'),
      facesReady: await faceStore.facesStatusCount(SyntheticStatus.READY),
      facesFailed: await faceStore.facesStatusCount(SyntheticStatus.FAILED)
    };
  };
};

export { type SyntheticJobStatus };
