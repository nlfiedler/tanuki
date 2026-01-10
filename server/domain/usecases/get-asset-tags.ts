//
// Copyright (c) 2026 Nathan Fiedler
//
import assert from 'node:assert';
import { AttributeCount } from 'tanuki/server/domain/entities/attributes.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';

export default ({
  recordRepository
}: {
  recordRepository: RecordRepository;
}) => {
  assert.ok(recordRepository, 'record repository must be defined');
  /**
   * Retrieve the union of tags for the given assets.
   *
   * @returns attribute counts of tags for the given assets.
   */
  return async (assets: string[]): Promise<AttributeCount[]> => {
    const counts: Map<string, number> = new Map();
    for (const assetId of assets) {
      const asset = await recordRepository.getAssetById(assetId);
      for (const tag of asset!.tags) {
        counts.set(tag, (counts.get(tag) ?? 0) + 1);
      }
    }
    const results: AttributeCount[] = [];
    for (const [key, value] of counts) {
      results.push(new AttributeCount(key, value));
    }
    return results;
  };
};
