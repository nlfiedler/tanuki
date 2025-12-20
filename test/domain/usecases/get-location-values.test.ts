//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, mock, test } from 'bun:test';
import { Location } from 'tanuki/server/domain/entities/location.ts';
import GetLocationValues from 'tanuki/server/domain/usecases/get-location-values.ts';
import { recordRepositoryMock } from './mocking.ts';

describe('GetLocationValues use case', function () {
  test('should do nothing when no locations', async function () {
    // arrange
    const mockRecordRepository = recordRepositoryMock({});
    const usecase = GetLocationValues({
      recordRepository: mockRecordRepository
    });
    // act
    const { labels, cities, regions } = await usecase();
    // assert
    expect(labels).toHaveLength(0);
    expect(cities).toHaveLength(0);
    expect(regions).toHaveLength(0);
    expect(mockRecordRepository.rawLocations).toHaveBeenCalledTimes(1);
    mock.clearAllMocks();
  });

  test('should work for a single location', async function () {
    // arrange
    const mockRecordRepository = recordRepositoryMock({
      rawLocations: mock(() =>
        Promise.resolve([Location.parse('beach; Oahu, Hawaii')])
      )
    });
    const usecase = GetLocationValues({
      recordRepository: mockRecordRepository
    });
    // act
    const { labels, cities, regions } = await usecase();
    // assert
    expect(labels).toHaveLength(1);
    expect(labels[0]).toEqual('beach');
    expect(cities).toHaveLength(1);
    expect(cities[0]).toEqual('Oahu');
    expect(regions).toHaveLength(1);
    expect(regions[0]).toEqual('Hawaii');
    expect(mockRecordRepository.rawLocations).toHaveBeenCalledTimes(1);
    mock.clearAllMocks();
  });

  test('should merge duplicate values for each location field', async function () {
    // arrange
    const mockRecordRepository = recordRepositoryMock({
      rawLocations: mock(() =>
        Promise.resolve([
          Location.parse('hotel; Oahu, Hawaii'),
          Location.parse('beach; Oahu, Hawaii'),
          Location.parse('beach; Alameda, California'),
          Location.parse('Alameda, California'),
          Location.parse('museum; Oakland, California'),
          Location.parse('beach; Kailua-Kona, Hawaii')
        ])
      )
    });
    const usecase = GetLocationValues({
      recordRepository: mockRecordRepository
    });
    // act
    const { labels, cities, regions } = await usecase();
    // assert
    expect(labels).toHaveLength(3);
    expect(labels.toSorted()).toEqual(['beach', 'hotel', 'museum']);
    expect(cities).toHaveLength(4);
    expect(cities.toSorted()).toEqual([
      'Alameda',
      'Kailua-Kona',
      'Oahu',
      'Oakland'
    ]);
    expect(regions).toHaveLength(2);
    expect(regions.toSorted()).toEqual(['California', 'Hawaii']);
    expect(mockRecordRepository.rawLocations).toHaveBeenCalledTimes(1);
    mock.clearAllMocks();
  });
});
