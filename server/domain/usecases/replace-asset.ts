//
// Copyright (c) 2026 Nathan Fiedler
//
import assert from 'node:assert';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { type BlobRepository } from 'tanuki/server/domain/repositories/blob-repository.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';

export default ({
  recordRepository,
  blobRepository
}: {
  recordRepository: RecordRepository;
  blobRepository: BlobRepository;
}) => {
  assert.ok(recordRepository, 'record repository must be defined');
  assert.ok(blobRepository, 'blob repository must be defined');
  /**
   * Read certain properties (tags, caption, location) from the old asset record
   * and copy to the new record, then delete the old record and the old blob.
   *
   * @param oldAssetId - identifier of asset to be removed.
   * @param newAssetId - identifier of replacement asset.
   * @returns the updated asset entity.
   */
  return async (oldAssetId: string, newAssetId: string): Promise<Asset> => {
    const oldAsset = await recordRepository.getAssetById(oldAssetId);
    if (oldAsset === null) {
      throw new Error(`no such asset: ${oldAssetId}`);
    }
    const newAsset = await recordRepository.getAssetById(newAssetId);
    if (newAsset === null) {
      throw new Error(`no such asset: ${newAssetId}`);
    }
    newAsset.tags = oldAsset.tags;
    newAsset.caption = oldAsset.caption;
    newAsset.location = oldAsset.location;
    await recordRepository.putAsset(newAsset);
    await recordRepository.deleteAsset(oldAssetId);
    await blobRepository.deleteBlob(oldAssetId);
    return newAsset;
  };
};
