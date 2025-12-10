//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import { AttributeCount } from 'tanuki/server/domain/entities/attributes.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';

export default ({
  recordRepository
}: {
  recordRepository: RecordRepository;
}) => {
  assert.ok(recordRepository, 'record repository must be defined');
  /**
   * Return all of the media types and their record counts.
   *
   * @returns media types and the number of occurrences.
   */
  return (): Promise<AttributeCount[]> => {
    return recordRepository.allMediaTypes();
  };
};
