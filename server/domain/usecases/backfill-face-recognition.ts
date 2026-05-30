//
// Copyright (c) 2026 Nathan Fiedler
//
import assert from 'node:assert';
import { SyntheticStatus } from 'tanuki/server/domain/entities/synthetic-data.ts';
import { type FaceStore } from 'tanuki/server/domain/repositories/face-store.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';
import { type SyntheticDetector } from 'tanuki/server/domain/services/synthetic-detector.ts';

export default ({
  recordRepository,
  faceStore,
  syntheticDetector
}: {
  recordRepository: RecordRepository;
  faceStore: FaceStore;
  syntheticDetector: SyntheticDetector;
}) => {
  assert.ok(recordRepository, 'record repository must be defined');
  assert.ok(faceStore, 'face store must be defined');
  assert.ok(syntheticDetector, 'synthetic detector must be defined');
  /**
   * Walk every image asset and enqueue a background `faces` job (at backfill
   * priority) for each one that lacks current face data and is not already
   * queued. An asset needs work when its faces status is not `READY` (never
   * processed, or failed), or when its stored faces were produced by a model
   * version other than the detector's current one (a model upgrade).
   *
   * Idempotent and resumable: assets already processed with the current model,
   * or that already have a faces job waiting, are skipped, so re-running does
   * not duplicate work.
   *
   * @param force - re-enqueue every image regardless of its current faces
   *   status (still skipping assets that already have a job queued). Useful
   *   after switching detectors, since both backends share the same
   *   `model_version` and so nothing looks "stale" to the default path.
   * @returns the number of assets enqueued.
   */
  return async (force = false): Promise<number> => {
    const currentVersion = syntheticDetector.faceModelVersion();
    let enqueued = 0;
    let cursor = null;
    while (true) {
      const [assets, next] = await recordRepository.fetchAssets(cursor, 1024);
      if (assets.length === 0) break;
      const images = assets.filter((asset) =>
        asset.mediaType.startsWith('image/')
      );
      if (images.length > 0) {
        const ids = images.map((asset) => asset.key);
        // The status / version maps only matter for the selective (non-force)
        // path; skip those reads entirely when forcing a full reprocess.
        const statusMap = force
          ? null
          : await faceStore.fetchFacesStatus(ids);
        const versionsMap = force
          ? null
          : await faceStore.modelVersionsByAssets(ids);
        for (const asset of images) {
          if (!force) {
            const ready = statusMap!.get(asset.key) === SyntheticStatus.READY;
            const versions = versionsMap!.get(asset.key);
            // Stale when any stored face used a different model version.
            const stale =
              versions !== undefined &&
              [...versions].some((v) => v !== currentVersion);
            if (ready && !stale) continue;
          }
          if (await faceStore.hasPendingJob(asset.key, 'faces')) continue;
          await faceStore.enqueueJob(asset.key, 'faces');
          enqueued++;
        }
      }
      cursor = next;
    }
    return enqueued;
  };
};
