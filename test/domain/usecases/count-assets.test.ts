//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, mock, test } from 'bun:test';
import CountAssets from 'tanuki/server/domain/usecases/count-assets.ts';
import { recordRepositoryMock } from './mocking.ts';

describe('CountAssets use case', function () {
  test('should return number of assets', async function () {
    // arrange
    const mockRecordRepository = recordRepositoryMock({
      countAssets: mock(() => Promise.resolve(101))
    });
    const usecase = CountAssets({
      recordRepository: mockRecordRepository
    });
    // act
    const count = await usecase();
    // assert
    expect(count).toEqual(101);
    expect(mockRecordRepository.countAssets).toHaveBeenCalledTimes(1);
    mock.clearAllMocks();
  });
});
