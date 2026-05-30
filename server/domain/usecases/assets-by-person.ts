//
// Copyright (c) 2026 Nathan Fiedler
//
import assert from 'node:assert';
import { type Asset } from 'tanuki/server/domain/entities/asset.ts';
import { SearchResult } from 'tanuki/server/domain/entities/search.ts';
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
   * Page the assets in which a person appears. The face store pages the asset
   * ids (most-recently-detected first) and returns the total; we materialize
   * the page into search results via the record store. Results preserve the
   * store's page order — pagination is by detection recency, not asset date,
   * since the face store does not know asset dates.
   *
   * @param personId - the person whose assets to return.
   * @param offset - zero-based page offset.
   * @param limit - maximum results to return.
   * @returns the page of search results and the total asset count.
   */
  return async (
    personId: string,
    offset: number,
    limit: number
  ): Promise<{ results: SearchResult[]; total: number }> => {
    const { ids, total } = await faceStore.assetIdsByPerson(
      personId,
      offset,
      limit
    );
    // Materialize the page concurrently (one round-trip each, in parallel)
    // rather than serially; Promise.all preserves the store's order. Ids whose
    // asset has since been removed come back null and are dropped.
    const assets = await Promise.all(
      ids.map((id) => recordRepository.getAssetById(id))
    );
    const results = assets
      .filter((asset): asset is Asset => asset !== null)
      .map((asset) => SearchResult.fromAsset(asset));
    return { results, total };
  };
};
