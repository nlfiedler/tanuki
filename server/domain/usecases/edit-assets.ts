//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import { type Operation } from 'tanuki/server/domain/entities/edit.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';

export default ({
  recordRepository
}: {
  recordRepository: RecordRepository;
}) => {
  assert.ok(recordRepository, 'record repository must be defined');
  /**
   * Makes changes to multiple assets at one time. The inputs include the set of
   * asset identifiers and a list of operations to be performed on those assets.
   *
   * @returns number of assets that were modified.
   */
  return async (assets: string[], mods: Operation[]): Promise<number> => {
    let fixedCount = 0;
    for (const assetId of assets) {
      const asset = await recordRepository.getAssetById(assetId);
      if (asset === null) {
        throw new Error(`asset ${assetId} not found`);
      }
      let modded = false;
      for (const mod of mods) {
        if (mod.perform(asset)) {
          modded = true;
        }
      }
      if (modded) {
        recordRepository.putAsset(asset);
        fixedCount++;
      }
    }
    // if (fixedCount > 0) {
    //   searchRepository.clear();
    // }
    return fixedCount;
  };
};
