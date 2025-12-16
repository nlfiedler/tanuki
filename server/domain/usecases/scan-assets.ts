//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';
import {
  SearchResult,
  SortField,
  SortOrder
} from 'tanuki/server/domain/entities/search.ts';
import { parse } from './query.ts';
import * as helpers from './helpers.ts';

export default ({
  recordRepository
}: {
  recordRepository: RecordRepository;
}) => {
  assert.ok(recordRepository, 'record repository must be defined');
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
    if (query.length === 0) {
      return [];
    }
    const cons = await parse(query);
    let cursor = null;
    const results: SearchResult[] = [];
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
    helpers.sortSearchResults(results, sortField ?? null, sortOrder ?? null);
    return results;
  };
};
