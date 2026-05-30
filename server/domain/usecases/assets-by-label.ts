//
// Copyright (c) 2026 Nathan Fiedler
//
import assert from 'node:assert';
import { type SearchResult } from 'tanuki/server/domain/entities/search.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';

export default ({
  recordRepository
}: {
  recordRepository: RecordRepository;
}) => {
  assert.ok(recordRepository, 'record repository must be defined');
  /**
   * Return every asset whose `synthetic.primaryLabel` matches the given label.
   * Results come back unsorted; the caller (resolver) paginates and orders.
   *
   * @param label - the curated display label to match (case-insensitive).
   */
  return async (label: string): Promise<SearchResult[]> => {
    return recordRepository.queryByLabel(label);
  };
};
