//
// Copyright (c) 2026 Nathan Fiedler
//
import DataLoader from 'dataloader';
import { AssetMetadata } from 'tanuki/server/domain/entities/asset-metadata.ts';
import {
  SyntheticData,
  SyntheticStatus
} from 'tanuki/server/domain/entities/synthetic-data.ts';
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

/**
 * Per-request `SyntheticData` loader. Coalesces multiple `synthetic` field
 * resolutions into a single batched call to the record repository.
 */
export function createSyntheticLoader(
  recordRepository: RecordRepository
): DataLoader<string, SyntheticData | null> {
  return new DataLoader<string, SyntheticData | null>(async (assetIds) => {
    const map = await recordRepository.fetchSynthetic([...assetIds]);
    return assetIds.map((id) => map.get(id) ?? null);
  });
}

/**
 * Per-request `SyntheticStatus` loader. Returns `PENDING` for any id that the
 * repository does not have a row for, matching the contract of
 * `fetchSyntheticStatus`.
 */
export function createSyntheticStatusLoader(
  recordRepository: RecordRepository
): DataLoader<string, SyntheticStatus> {
  return new DataLoader<string, SyntheticStatus>(async (assetIds) => {
    const map = await recordRepository.fetchSyntheticStatus([...assetIds]);
    return assetIds.map((id) => map.get(id) ?? SyntheticStatus.PENDING);
  });
}

export type GraphQLContext = {
  metadataLoader: DataLoader<string, AssetMetadata | null>;
  syntheticLoader: DataLoader<string, SyntheticData | null>;
  syntheticStatusLoader: DataLoader<string, SyntheticStatus>;
};
