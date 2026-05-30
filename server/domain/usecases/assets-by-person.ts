//
// Copyright (c) 2026 Nathan Fiedler
//
import assert from 'node:assert';
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
   * ids (newest detection first); we materialize each into a search result via
   * the record store. The store's total is returned alongside so the resolver
   * can paginate without loading every asset.
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
    const results: SearchResult[] = [];
    for (const id of ids) {
      const asset = await recordRepository.getAssetById(id);
      // skip ids whose asset has since been removed (defensive)
      if (asset !== null) results.push(SearchResult.fromAsset(asset));
    }
    return { results, total };
  };
};
