//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, mock, test } from 'bun:test';
import { Location } from 'tanuki/server/domain/entities/location.ts';
import { PendingParams, SearchResult } from 'tanuki/server/domain/entities/search.ts';
import FindPending from 'tanuki/server/domain/usecases/find-pending.ts';
import { recordRepositoryMock } from './mocking.ts';

describe('FindPending use case', function () {
  test('should find nothing when zero assets', async function () {
    // arrange
    const mockRecordRepository = recordRepositoryMock({});
    const usecase = FindPending({ recordRepository: mockRecordRepository });
    // act
    const params = new PendingParams();
    const actual = await usecase(params);
    // assert
    expect(actual).toHaveLength(0);
    expect(mockRecordRepository.queryNewborn).toHaveBeenCalledTimes(1);
    mock.clearAllMocks();
  });

  test('should return everything if no after date given', async function () {
    //
    // this is a fake test since the mock is doing the filtering
    //
    // arrange
    const results = [
      new SearchResult(
        'monday1',
        'img_1234.jpg',
        'image/jpeg',
        Location.parse('Oahu, Hawaii'),
        new Date(2000, 1, 1, 0, 0)
      ),
      new SearchResult(
        'monday2',
        'img_1234.jpg',
        'image/jpeg',
        Location.parse('Oahu, Hawaii'),
        new Date(2001, 1, 1, 0, 0)
      ),
      new SearchResult(
        'monday3',
        'img_1234.jpg',
        'image/jpeg',
        Location.parse('Oahu, Hawaii'),
        new Date(2002, 1, 1, 0, 0)
      ),
      new SearchResult(
        'monday4',
        'img_1234.jpg',
        'image/jpeg',
        Location.parse('Oahu, Hawaii'),
        new Date(2003, 1, 1, 0, 0)
      ),
      new SearchResult(
        'monday5',
        'img_1234.jpg',
        'image/jpeg',
        Location.parse('Oahu, Hawaii'),
        new Date(2004, 1, 1, 0, 0)
      )
    ];
    const mockRecordRepository = recordRepositoryMock({
      queryNewborn: mock(() => Promise.resolve(results))
    });
    const usecase = FindPending({ recordRepository: mockRecordRepository });
    // act
    const params = new PendingParams();
    const actual = await usecase(params);
    // assert
    expect(actual).toHaveLength(5);
    expect(actual.some((l) => l.assetId == 'monday1')).toBeTrue();
    expect(actual.some((l) => l.assetId == 'monday2')).toBeTrue();
    expect(actual.some((l) => l.assetId == 'monday3')).toBeTrue();
    expect(actual.some((l) => l.assetId == 'monday4')).toBeTrue();
    expect(actual.some((l) => l.assetId == 'monday5')).toBeTrue();
    mock.clearAllMocks();
  });

  test('should filter results by date after', async function () {
    //
    // this is a fake test since the mock is doing the filtering
    //
    // arrange
    const results = [
      new SearchResult(
        'monday3',
        'img_1234.jpg',
        'image/jpeg',
        Location.parse('Oahu, Hawaii'),
        new Date(2002, 1, 1, 0, 0)
      ),
      new SearchResult(
        'monday4',
        'img_1234.jpg',
        'image/jpeg',
        Location.parse('Oahu, Hawaii'),
        new Date(2003, 1, 1, 0, 0)
      ),
      new SearchResult(
        'monday5',
        'img_1234.jpg',
        'image/jpeg',
        Location.parse('Oahu, Hawaii'),
        new Date(2004, 1, 1, 0, 0)
      )
    ];
    const mockRecordRepository = recordRepositoryMock({
      queryNewborn: mock(() => Promise.resolve(results))
    });
    const usecase = FindPending({ recordRepository: mockRecordRepository });
    // act
    const params = new PendingParams().setAfterDate(new Date(2002, 0, 1, 0, 0));
    const actual = await usecase(params);
    // assert
    expect(actual).toHaveLength(3);
    expect(actual.some((l) => l.assetId == 'monday3')).toBeTrue();
    expect(actual.some((l) => l.assetId == 'monday4')).toBeTrue();
    expect(actual.some((l) => l.assetId == 'monday5')).toBeTrue();
    mock.clearAllMocks();
  });
});
