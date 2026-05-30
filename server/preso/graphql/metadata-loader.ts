//
// Copyright (c) 2026 Nathan Fiedler
//
import DataLoader from 'dataloader';
import { AssetMetadata } from 'tanuki/server/domain/entities/asset-metadata.ts';
import {
  type PersonSummary
} from 'tanuki/server/domain/entities/face.ts';
import {
  SyntheticData,
  SyntheticStatus
} from 'tanuki/server/domain/entities/synthetic-data.ts';
import { type FaceStore } from 'tanuki/server/domain/repositories/face-store.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';

/**
 * Combine a labels status and a faces status into the single `syntheticStatus`
 * a client sees: the worse of the two (FAILED ≻ PENDING ≻ READY), so an asset
 * is only READY once both pipelines have finished, and surfaces FAILED if
 * either gave up.
 */
function worseStatus(
  a: SyntheticStatus,
  b: SyntheticStatus
): SyntheticStatus {
  const severity = (s: SyntheticStatus): number => {
    if (s === SyntheticStatus.FAILED) return 2;
    if (s === SyntheticStatus.PENDING) return 1;
    return 0;
  };
  return severity(a) >= severity(b) ? a : b;
}

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
 * Per-request `SyntheticStatus` loader. The surfaced status is the worse of
 * the labels status (on the asset record) and the faces status (in the face
 * store); each defaults to `PENDING` when no row exists. One batched call per
 * store, regardless of page size.
 */
export function createSyntheticStatusLoader(
  recordRepository: RecordRepository,
  faceStore: FaceStore
): DataLoader<string, SyntheticStatus> {
  return new DataLoader<string, SyntheticStatus>(async (assetIds) => {
    const ids = [...assetIds];
    const [labels, faces] = await Promise.all([
      recordRepository.fetchSyntheticStatus(ids),
      faceStore.fetchFacesStatus(ids)
    ]);
    return assetIds.map((id) =>
      worseStatus(
        labels.get(id) ?? SyntheticStatus.PENDING,
        faces.get(id) ?? SyntheticStatus.PENDING
      )
    );
  });
}

/**
 * Per-request loader resolving the people in each asset, batched for the
 * `synthetic.people` field. Returns an empty array for assets with no faces.
 */
export function createPeopleLoader(
  faceStore: FaceStore
): DataLoader<string, PersonSummary[]> {
  return new DataLoader<string, PersonSummary[]>(async (assetIds) => {
    const map = await faceStore.fetchPeopleByAssetIds([...assetIds]);
    return assetIds.map((id) => map.get(id) ?? []);
  });
}

export type GraphQLContext = {
  metadataLoader: DataLoader<string, AssetMetadata | null>;
  syntheticLoader: DataLoader<string, SyntheticData | null>;
  syntheticStatusLoader: DataLoader<string, SyntheticStatus>;
  peopleLoader: DataLoader<string, PersonSummary[]>;
};
