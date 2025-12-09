//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import * as helpers from './helpers.ts';
import { PendingParams } from 'tanuki/server/domain/entities/SearchParams.ts';
import { SearchResult } from 'tanuki/server/domain/entities/SearchResult.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/RecordRepository.ts';

export default ({
  recordRepository
}: {
  recordRepository: RecordRepository;
}) => {
  assert.ok(recordRepository, 'record repository must be defined');
  /**
   * Query for assets that do not have any tags, caption, and location label.
   *
   * @param params - pending parameters.
   * @returns array of results containing selected fields from asset records.
   */
  return async (params: PendingParams): Promise<SearchResult[]> => {
    const results = await recordRepository.queryNewborn(
      params.afterDate || new Date(-271821, 3, 20)
    );
    helpers.sortSearchResults(results, params.sortField, params.sortOrder);
    return results;
  };
};
