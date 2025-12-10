//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';

export default ({
  recordRepository
}: {
  recordRepository: RecordRepository;
}) => {
  assert.ok(recordRepository, 'record repository must be defined');
  /**
   * Count the number of records in the record repository.
   *
   * @returns number of assets in the database.
   */
  return (): Promise<number> => {
    return recordRepository.countAssets();
  };
};
