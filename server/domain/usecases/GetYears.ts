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
   * Return all of the years for which there are assets and their record counts.
   *
   * @returns years and the number of occurrences.
   */
  return (): Promise<AttributeCount[]> => {
    return recordRepository.allYears();
  };
};
