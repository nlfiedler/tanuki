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
   * `FAILED`, resetting each back to `PENDING`. Phase 1 only has `labels`
   * jobs, so a `kind` of `faces` is a no-op until that pipeline lands.
   *
   * Idempotent: an asset already carrying a queued labels job is left alone.
   *
   * @param kind - restrict to one pipeline (`labels` / `faces`), or null for all.
   * @returns the number of assets re-enqueued.
   */
  return async (kind?: string | null): Promise<number> => {
    // nothing other than labels can be retried in Phase 1
    if (kind && kind !== 'labels') return 0;
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
  };
};
