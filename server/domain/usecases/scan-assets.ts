//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import { type FaceStore } from 'tanuki/server/domain/repositories/face-store.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';
import { type SearchRepository } from 'tanuki/server/domain/repositories/search-repository.ts';
import {
  SearchResult,
  SortField,
  SortOrder
} from 'tanuki/server/domain/entities/search.ts';
import { parse } from './query.ts';
import * as helpers from './helpers.ts';

/** Upper bound on a person's assets resolved for a `person:` query term. */
const PERSON_ASSET_LIMIT = 1_000_000;

export default ({
  recordRepository,
  searchRepository,
  faceStore
}: {
  recordRepository: RecordRepository;
  searchRepository: SearchRepository;
  faceStore: FaceStore;
}) => {
  assert.ok(recordRepository, 'record repository must be defined');
  assert.ok(searchRepository, 'search repository must be defined');
  assert.ok(faceStore, 'face store must be defined');
  /**
   * Scan all assets in the database matching each record against multiple
   * criteria with optional boolean operators and grouping.
   *
   * @param query - query string for finding matching assets.
   * @param sortField - field on which to sort.
   * @param sortOrder - order for sorting (ascending or descending).
   * @returns array of search results.
   */
  return async (
    query: string,
    sortField?: SortField,
    sortOrder?: SortOrder
  ): Promise<SearchResult[]> => {
    if (query.trim().length === 0) {
      return [];
    }
    let results: SearchResult[] = [];
    const cached = await searchRepository.get(query);
    if (cached) {
      results = cached;
    } else {
      // Resolve `person:<token>` terms against the face store. The token may be
      // an opaque person id OR a (case-insensitive) name, so treat it as both:
      // union the assets of the person with that id and of every person with
      // that name. A token that is only one of the two contributes the empty
      // set from the other lookup, so `person:<uuid>` and `person:"Alice"` both
      // work — and a name shared by several clusters matches all of them.
      const cons = await parse(query, async (token) => {
        const personIds = new Set<string>([token]);
        for (const id of await faceStore.personIdsByName(token)) {
          personIds.add(id);
        }
        const assetIds = new Set<string>();
        for (const personId of personIds) {
          const { ids } = await faceStore.assetIdsByPerson(
            personId,
            0,
            PERSON_ASSET_LIMIT
          );
          for (const id of ids) assetIds.add(id);
        }
        return assetIds;
      });
      let cursor = null;
      while (true) {
        const [assets, next] = await recordRepository.fetchAssets(cursor, 1024);
        for (const entry of assets) {
          if (cons.matches(entry)) {
            results.push(SearchResult.fromAsset(entry));
          }
        }
        if (assets.length === 0) {
          break;
        }
        cursor = next;
      }
      await searchRepository.put(query, results);
    }
    helpers.sortSearchResults(results, sortField ?? null, sortOrder ?? null);
    return results;
  };
};
