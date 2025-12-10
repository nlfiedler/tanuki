//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import * as helpers from './helpers.ts';
import {
  PendingParams,
  SearchResult
} from 'tanuki/server/domain/entities/search.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';

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
      params.afterDate || new Date(-271_821, 3, 20)
    );
    helpers.sortSearchResults(results, params.sortField, params.sortOrder);
    return results;
  };
};
