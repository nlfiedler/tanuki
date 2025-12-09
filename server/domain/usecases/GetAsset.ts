//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import { Asset } from 'tanuki/server/domain/entities/Asset.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/RecordRepository.ts';

export default ({
  recordRepository
}: {
  recordRepository: RecordRepository;
}) => {
  assert.ok(recordRepository, 'record repository must be defined');
  /**
   * Retrieve an asset entity from the record repository.
   *
   * @returns asset entity, or null if not found.
   */
  return async (assetId: string): Promise<Asset | null> => {
    return await recordRepository.getAssetById(assetId);
  };
};
