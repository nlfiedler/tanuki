//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import { AttributeCount } from 'tanuki/server/domain/entities/AttributeCount.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/RecordRepository.ts';

/**
 * Return all of the asset tags and their record counts.
 * 
 * @returns tags and the number of occurrences.
 */
export default ({ recordRepository }: { recordRepository: RecordRepository; }) => {
  assert.ok(recordRepository, 'record repository must be defined');
  return (): Promise<AttributeCount[]> => {
    return recordRepository.allTags();
  };
};
