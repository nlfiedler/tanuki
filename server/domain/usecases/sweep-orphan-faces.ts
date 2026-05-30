//
// Copyright (c) 2026 Nathan Fiedler
//
import assert from 'node:assert';
import { type FaceStore } from 'tanuki/server/domain/repositories/face-store.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';
import logger from 'tanuki/server/logger.ts';

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
   * Defensive cross-store consistency sweep: face rows reference assets in a
   * separate database with no SQL cascade, so this finds every distinct
   * `asset_id` that still has faces, checks whether the asset still exists, and
   * removes the faces (and any now-empty person rows) for those that don't.
   *
   * The asset-deletion use case already cleans up on the normal path, so this
   * should usually find nothing; it exists to catch bugs that bypass that path.
   *
   * @returns the number of assets whose orphaned faces were removed.
   */
  return async (): Promise<number> => {
    const assetIds = await faceStore.allFaceAssetIds();
    let removed = 0;
    for (const assetId of assetIds) {
      const asset = await recordRepository.getAssetById(assetId);
      if (asset === null) {
        await faceStore.deleteByAssetId(assetId);
        removed++;
      }
    }
    if (removed > 0) {
      logger.info(`orphan sweep removed faces for ${removed} missing asset(s)`);
    }
    return removed;
  };
};
