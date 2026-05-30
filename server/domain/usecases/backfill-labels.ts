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
   * Walk every image asset and enqueue a background `labels` job (at backfill
   * priority) for each one that lacks label data and is not already queued.
   *
   * Idempotent and resumable: assets that are already `READY`, or that already
   * have a labels job waiting, are skipped, so re-running does not duplicate
   * work.
   *
   * @returns the number of assets enqueued.
   */
  return async (): Promise<number> => {
    let enqueued = 0;
    let cursor = null;
    while (true) {
      const [assets, next] = await recordRepository.fetchAssets(cursor, 1024);
      if (assets.length === 0) break;
      const images = assets.filter((asset) =>
        asset.mediaType.startsWith('image/')
      );
      if (images.length > 0) {
        const statusMap = await recordRepository.fetchSyntheticStatus(
          images.map((asset) => asset.key)
        );
        for (const asset of images) {
          // already labelled — nothing to do
          if (statusMap.get(asset.key) === SyntheticStatus.READY) continue;
          // a job is already waiting (e.g. a recent live import) — don't dup
          if (await faceStore.hasPendingJob(asset.key, 'labels')) continue;
          await faceStore.enqueueJob(asset.key, 'labels');
          enqueued++;
        }
      }
      cursor = next;
    }
    return enqueued;
  };
};
