//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import { Location } from 'tanuki/server/domain/entities/Location.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/RecordRepository.ts';

export default ({
  recordRepository
}: {
  recordRepository: RecordRepository;
}) => {
  assert.ok(recordRepository, 'record repository must be defined');
  /**
   * Return all of the location records in the record repository. A location
   * record has a label, city, and region together in a single unit.
   *
   * @returns list of unique location records.
   */
  return (): Promise<Location[]> => {
    return recordRepository.rawLocations();
  };
};
