//
// Copyright (c) 2026 Nathan Fiedler
//
import DataLoader from 'dataloader';
import { AssetMetadata } from 'tanuki/server/domain/entities/asset-metadata.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';

/**
 * Per-request `AssetMetadata` loader. Coalesces multiple `metadata` field
 * resolutions in a single GraphQL operation into one batched repository call.
 */
export function createMetadataLoader(
  recordRepository: RecordRepository
): DataLoader<string, AssetMetadata | null> {
  return new DataLoader<string, AssetMetadata | null>(async (assetIds) => {
    const map = await recordRepository.fetchMetadata([...assetIds]);
    return assetIds.map((id) => map.get(id) ?? null);
  });
}

export type GraphQLContext = {
  metadataLoader: DataLoader<string, AssetMetadata | null>;
};
