//
// Copyright (c) 2026 Nathan Fiedler
//
import assert from 'node:assert';
import { SyntheticStatus } from 'tanuki/server/domain/entities/synthetic-data.ts';
import { type FaceStore } from 'tanuki/server/domain/repositories/face-store.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';

export default ({
  recordRepository,
  faceStore
}: {
  recordRepository: RecordRepository;
  faceStore: FaceStore;
}) => {
  assert.ok(recordRepository, 'record repository must be defined');
  assert.ok(faceStore, 'face store must be defined');
  /**
   * Re-enqueue extraction jobs for every asset whose synthetic status is
   * `FAILED`, resetting each back to `PENDING`. Labels status lives on the
   * asset record; faces status lives in the face store, so the two kinds are
   * scanned from their respective stores.
   *
   * Idempotent: an asset already carrying a queued job for that kind is left
   * alone.
   *
   * @param kind - restrict to one pipeline (`labels` / `faces`), or null for all.
   * @returns the number of assets re-enqueued.
   */
  return async (kind?: string | null): Promise<number> => {
    if (kind && kind !== 'labels' && kind !== 'faces') return 0;
    let requeued = 0;
    if (!kind || kind === 'labels') requeued += await retryLabels();
    if (!kind || kind === 'faces') requeued += await retryFaces();
    return requeued;
  };

  /** Re-enqueue labels jobs for assets whose labels status is FAILED. */
  async function retryLabels(): Promise<number> {
    let requeued = 0;
    let cursor = null;
    while (true) {
      const [assets, next] = await recordRepository.fetchAssets(cursor, 1024);
      if (assets.length === 0) break;
      const statusMap = await recordRepository.fetchSyntheticStatus(
        assets.map((asset) => asset.key)
      );
      for (const asset of assets) {
        if (statusMap.get(asset.key) !== SyntheticStatus.FAILED) continue;
        if (await faceStore.hasPendingJob(asset.key, 'labels')) continue;
        // Reset status FIRST: enqueuing before this opens a race where a
        // worker can claim and write READY before the PENDING reset runs,
        // wiping the successful result.
        await recordRepository.setSynthetic(
          asset.key,
          null,
          SyntheticStatus.PENDING
        );
        await faceStore.enqueueJob(asset.key, 'labels');
        requeued++;
      }
      cursor = next;
    }
    return requeued;
  }

  /** Re-enqueue faces jobs for assets whose faces status is FAILED. */
  async function retryFaces(): Promise<number> {
    let requeued = 0;
    const failed = await faceStore.assetIdsWithFacesStatus(
      SyntheticStatus.FAILED
    );
    for (const assetId of failed) {
      if (await faceStore.hasPendingJob(assetId, 'faces')) continue;
      // Reset to PENDING before enqueuing (same race rationale as labels).
      await faceStore.setFacesStatus(assetId, SyntheticStatus.PENDING);
      await faceStore.enqueueJob(assetId, 'faces');
      requeued++;
    }
    return requeued;
  }
};
