//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import { AttributeCount } from 'tanuki/server/domain/entities/AttributeCount.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/RecordRepository.ts';

export default ({
  recordRepository
}: {
  recordRepository: RecordRepository;
}) => {
  assert.ok(recordRepository, 'record repository must be defined');
  /**
   * Return all of the location parts in the record repository. Location parts are
   * formed by splitting the label, city, and region from the location records.
   *
   * @returns list of location parts and associated counts.
   */
  return (): Promise<AttributeCount[]> => {
    return recordRepository.allLocations();
  };
};
