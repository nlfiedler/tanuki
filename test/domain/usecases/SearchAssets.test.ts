//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, mock, test } from "bun:test";
import { Location } from 'tanuki/server/domain/entities/Location.ts';
import { SearchResult } from 'tanuki/server/domain/entities/SearchResult.ts';
import { SearchParams, SortOrder, SortField } from 'tanuki/server/domain/entities/SearchParams.ts';
import SearchAssets from 'tanuki/server/domain/usecases/SearchAssets.ts';
import { recordRepositoryMock } from './mocking.ts';

describe('SearchAssets use case', function () {
  test('should do nothing when empty search params', async function () {
    // arrange
    const mockRecordRepository = recordRepositoryMock({});
    const usecase = SearchAssets({ recordRepository: mockRecordRepository });
    // act
    const params = new SearchParams();
    const actual = await usecase(params);
    // assert
    expect(actual).toHaveLength(0);
    expect(mockRecordRepository.queryByTags).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryByLocations).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryByMediaType).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryBeforeDate).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryAfterDate).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryDateRange).toHaveBeenCalledTimes(0);
    mock.clearAllMocks();
  });

  test('should find nothing when zero assets', async function () {
    // arrange
    const mockRecordRepository = recordRepositoryMock({});
    const usecase = SearchAssets({ recordRepository: mockRecordRepository });
    // act
    const params = new SearchParams().addTag('cat');
    const actual = await usecase(params);
    // assert
    expect(actual).toHaveLength(0);
    expect(mockRecordRepository.queryByTags).toHaveBeenCalledTimes(1);
    mock.clearAllMocks();
  });

  test('should search by tag when given tags', async function () {
    // arrange
    const results = [
      new SearchResult('monday2', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date()),
    ];
    const mockRecordRepository = recordRepositoryMock({
      queryByTags: mock(() => Promise.resolve(results)),
    });
    const usecase = SearchAssets({ recordRepository: mockRecordRepository });
    // act
    const params = new SearchParams().addTag('cat');
    const actual = await usecase(params);
    // assert
    expect(actual).toHaveLength(1);
    expect(actual[0]?.assetId).toEqual('monday2');
    expect(mockRecordRepository.queryByTags).toHaveBeenCalledTimes(1);
    expect(mockRecordRepository.queryByLocations).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryByMediaType).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryBeforeDate).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryAfterDate).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryDateRange).toHaveBeenCalledTimes(0);
    mock.clearAllMocks();
  });

  test('should search by date range when given after and before dates', async function () {
    // arrange
    const results = [
      new SearchResult('monday2', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date()),
    ];
    const mockRecordRepository = recordRepositoryMock({
      queryDateRange: mock(() => Promise.resolve(results)),
    });
    const usecase = SearchAssets({ recordRepository: mockRecordRepository });
    // act
    const params = new SearchParams().setBeforeDate(new Date(2008, 5, 1, 0, 0)).setAfterDate(new Date(2008, 4, 1, 0, 0));
    const actual = await usecase(params);
    // assert
    expect(actual).toHaveLength(1);
    expect(actual[0]?.assetId).toEqual('monday2');
    expect(mockRecordRepository.queryByTags).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryByLocations).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryByMediaType).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryBeforeDate).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryAfterDate).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryDateRange).toHaveBeenCalledTimes(1);
    mock.clearAllMocks();
  });

  test('should search by dates before a date given', async function () {
    // arrange
    const results = [
      new SearchResult('monday2', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date()),
    ];
    const mockRecordRepository = recordRepositoryMock({
      queryBeforeDate: mock(() => Promise.resolve(results)),
    });
    const usecase = SearchAssets({ recordRepository: mockRecordRepository });
    // act
    const params = new SearchParams().setBeforeDate(new Date(2008, 5, 1, 0, 0));
    const actual = await usecase(params);
    // assert
    expect(actual).toHaveLength(1);
    expect(actual[0]?.assetId).toEqual('monday2');
    expect(mockRecordRepository.queryByTags).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryByLocations).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryByMediaType).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryBeforeDate).toHaveBeenCalledTimes(1);
    expect(mockRecordRepository.queryAfterDate).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryDateRange).toHaveBeenCalledTimes(0);
    mock.clearAllMocks();
  });

  test('should search by dates after a given date', async function () {
    // arrange
    const results = [
      new SearchResult('monday2', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date()),
    ];
    const mockRecordRepository = recordRepositoryMock({
      queryAfterDate: mock(() => Promise.resolve(results)),
    });
    const usecase = SearchAssets({ recordRepository: mockRecordRepository });
    // act
    const params = new SearchParams().setAfterDate(new Date(2008, 4, 1, 0, 0));
    const actual = await usecase(params);
    // assert
    expect(actual).toHaveLength(1);
    expect(actual[0]?.assetId).toEqual('monday2');
    expect(mockRecordRepository.queryByTags).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryByLocations).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryByMediaType).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryBeforeDate).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryAfterDate).toHaveBeenCalledTimes(1);
    expect(mockRecordRepository.queryDateRange).toHaveBeenCalledTimes(0);
    mock.clearAllMocks();
  });

  test('should search by location when given locations', async function () {
    // arrange
    const results = [
      new SearchResult('monday2', 'img_1234.jpg', 'image/jpeg', Location.parse('Paris, Texas'), new Date()),
    ];
    const mockRecordRepository = recordRepositoryMock({
      queryByLocations: mock(() => Promise.resolve(results)),
    });
    const usecase = SearchAssets({ recordRepository: mockRecordRepository });
    // act
    const params = new SearchParams().addLocation('paris');
    const actual = await usecase(params);
    // assert
    expect(actual).toHaveLength(1);
    expect(actual[0]?.assetId).toEqual('monday2');
    expect(mockRecordRepository.queryByTags).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryByLocations).toHaveBeenCalledTimes(1);
    expect(mockRecordRepository.queryByMediaType).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryBeforeDate).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryAfterDate).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryDateRange).toHaveBeenCalledTimes(0);
    mock.clearAllMocks();
  });

  test('should search by media type when given a value', async function () {
    // arrange
    const results = [
      new SearchResult('monday2', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date()),
    ];
    const mockRecordRepository = recordRepositoryMock({
      queryByMediaType: mock(() => Promise.resolve(results)),
    });
    const usecase = SearchAssets({ recordRepository: mockRecordRepository });
    // act
    const params = new SearchParams().setMediaType('image/jpeg');
    const actual = await usecase(params);
    // assert
    expect(actual).toHaveLength(1);
    expect(actual[0]?.assetId).toEqual('monday2');
    expect(mockRecordRepository.queryByTags).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryByLocations).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryByMediaType).toHaveBeenCalledTimes(1);
    expect(mockRecordRepository.queryBeforeDate).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryAfterDate).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.queryDateRange).toHaveBeenCalledTimes(0);
    mock.clearAllMocks();
  });

  test('should filter results by date range', async function () {
    // arrange
    const results = [
      new SearchResult('monday1', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2000, 1, 1, 0, 0)),
      new SearchResult('monday2', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2001, 1, 1, 0, 0)),
      new SearchResult('monday3', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2002, 1, 1, 0, 0)),
      new SearchResult('monday4', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2003, 1, 1, 0, 0)),
      new SearchResult('monday5', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2004, 1, 1, 0, 0)),
    ];
    const mockRecordRepository = recordRepositoryMock({
      queryByTags: mock(() => Promise.resolve(results)),
    });
    const usecase = SearchAssets({ recordRepository: mockRecordRepository });
    // act
    const params = new SearchParams().addTag('cat').
      setAfterDate(new Date(2001, 0, 1, 0, 0)).
      setBeforeDate(new Date(2004, 0, 1, 0, 0));
    const actual = await usecase(params);
    // assert
    expect(actual).toHaveLength(3);
    expect(actual.some((l) => l.assetId == 'monday2')).toBeTrue();
    expect(actual.some((l) => l.assetId == 'monday3')).toBeTrue();
    expect(actual.some((l) => l.assetId == 'monday4')).toBeTrue();
    mock.clearAllMocks();
  });

  test('should filter results by date after', async function () {
    // arrange
    const results = [
      new SearchResult('monday1', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2000, 1, 1, 0, 0)),
      new SearchResult('monday2', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2001, 1, 1, 0, 0)),
      new SearchResult('monday3', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2002, 1, 1, 0, 0)),
      new SearchResult('monday4', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2003, 1, 1, 0, 0)),
      new SearchResult('monday5', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2004, 1, 1, 0, 0)),
    ];
    const mockRecordRepository = recordRepositoryMock({
      queryByTags: mock(() => Promise.resolve(results)),
    });
    const usecase = SearchAssets({ recordRepository: mockRecordRepository });
    // act
    const params = new SearchParams().addTag('cat').
      setAfterDate(new Date(2002, 0, 1, 0, 0));
    const actual = await usecase(params);
    // assert
    expect(actual).toHaveLength(3);
    expect(actual.some((l) => l.assetId == 'monday3')).toBeTrue();
    expect(actual.some((l) => l.assetId == 'monday4')).toBeTrue();
    expect(actual.some((l) => l.assetId == 'monday5')).toBeTrue();
    mock.clearAllMocks();
  });

  test('should filter results by date before', async function () {
    // arrange
    const results = [
      new SearchResult('monday1', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2000, 1, 1, 0, 0)),
      new SearchResult('monday2', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2001, 1, 1, 0, 0)),
      new SearchResult('monday3', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2002, 1, 1, 0, 0)),
      new SearchResult('monday4', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2003, 1, 1, 0, 0)),
      new SearchResult('monday5', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2004, 1, 1, 0, 0)),
    ];
    const mockRecordRepository = recordRepositoryMock({
      queryByTags: mock(() => Promise.resolve(results)),
    });
    const usecase = SearchAssets({ recordRepository: mockRecordRepository });
    // act
    const params = new SearchParams().addTag('cat').
      setBeforeDate(new Date(2003, 0, 1, 0, 0));
    const actual = await usecase(params);
    // assert
    expect(actual).toHaveLength(3);
    expect(actual.some((l) => l.assetId == 'monday1')).toBeTrue();
    expect(actual.some((l) => l.assetId == 'monday2')).toBeTrue();
    expect(actual.some((l) => l.assetId == 'monday3')).toBeTrue();
    mock.clearAllMocks();
  });

  test('should filter results by location', async function () {
    // arrange
    const results = [
      new SearchResult('monday1', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2000, 1, 1, 0, 0)),
      new SearchResult('monday2', 'img_1234.jpg', 'image/jpeg', Location.parse('Paris, Texas'), new Date(2001, 1, 1, 0, 0)),
      new SearchResult('monday3', 'img_1234.jpg', 'image/jpeg', Location.parse('Paris, France'), new Date(2002, 1, 1, 0, 0)),
      new SearchResult('monday4', 'img_1234.jpg', 'image/jpeg', Location.parse('Nice, France'), new Date(2003, 1, 1, 0, 0)),
      new SearchResult('monday5', 'img_1234.jpg', 'image/jpeg', Location.parse('Hilo, Hawaii'), new Date(2004, 1, 1, 0, 0)),
    ];
    const mockRecordRepository = recordRepositoryMock({
      queryByTags: mock(() => Promise.resolve(results)),
    });
    const usecase = SearchAssets({ recordRepository: mockRecordRepository });
    // act
    const single = await usecase(new SearchParams().addTag('cat').addLocation('paris'));
    // assert
    expect(single).toHaveLength(2);
    expect(single.some((l) => l.assetId == 'monday2')).toBeTrue();
    expect(single.some((l) => l.assetId == 'monday3')).toBeTrue();

    // act
    const multiple = await usecase(new SearchParams().addTag('cat').addLocation('paris').addLocation('france'));
    // assert
    expect(multiple).toHaveLength(1);
    expect(multiple.some((l) => l.assetId == 'monday3')).toBeTrue();
  });

  test('should filter results by media type', async function () {
    // arrange
    const results = [
      new SearchResult('monday1', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2000, 1, 1, 0, 0)),
      new SearchResult('monday2', 'img_1234.mov', 'video/mp4', Location.parse('Paris, Texas'), new Date(2001, 1, 1, 0, 0)),
      new SearchResult('monday3', 'img_1234.png', 'image/png', Location.parse('Paris, France'), new Date(2002, 1, 1, 0, 0)),
      new SearchResult('monday4', 'img_1234.jpg', 'image/jpeg', Location.parse('Nice, France'), new Date(2003, 1, 1, 0, 0)),
      new SearchResult('monday5', 'img_1234.mov', 'video/mp4', Location.parse('Hilo, Hawaii'), new Date(2004, 1, 1, 0, 0)),
    ];
    const mockRecordRepository = recordRepositoryMock({
      queryByTags: mock(() => Promise.resolve(results)),
    });
    const usecase = SearchAssets({ recordRepository: mockRecordRepository });
    // act
    const actual = await usecase(new SearchParams().addTag('cat').setMediaType('image/jpeg'));
    // assert
    expect(actual).toHaveLength(2);
    expect(actual.some((l) => l.assetId == 'monday1')).toBeTrue();
    expect(actual.some((l) => l.assetId == 'monday4')).toBeTrue();
  });

  test('should sort results by ascending date', async function () {
    // arrange
    const results = [
      new SearchResult('monday1', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2001, 1, 1, 0, 0)),
      new SearchResult('monday2', 'img_1234.mov', 'video/mp4', Location.parse('Paris, Texas'), new Date(2003, 1, 1, 0, 0)),
      new SearchResult('monday3', 'img_1234.png', 'image/png', Location.parse('Paris, France'), new Date(2002, 1, 1, 0, 0)),
      new SearchResult('monday4', 'img_1234.jpg', 'image/jpeg', Location.parse('Nice, France'), new Date(2000, 1, 1, 0, 0)),
      new SearchResult('monday5', 'img_1234.mov', 'video/mp4', Location.parse('Hilo, Hawaii'), new Date(2004, 1, 1, 0, 0)),
    ];
    const mockRecordRepository = recordRepositoryMock({
      queryByTags: mock(() => Promise.resolve(results)),
    });
    const usecase = SearchAssets({ recordRepository: mockRecordRepository });
    // act
    const actual = await usecase(new SearchParams().addTag('cat').setSortField(SortField.Date));
    // assert
    expect(actual).toHaveLength(5);
    expect(actual[0]?.assetId).toEqual('monday4');
    expect(actual[1]?.assetId).toEqual('monday1');
    expect(actual[2]?.assetId).toEqual('monday3');
    expect(actual[3]?.assetId).toEqual('monday2');
    expect(actual[4]?.assetId).toEqual('monday5');
  });

  test('should sort results by descending date', async function () {
    // arrange
    const results = [
      new SearchResult('monday1', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2001, 1, 1, 0, 0)),
      new SearchResult('monday2', 'img_1234.mov', 'video/mp4', Location.parse('Paris, Texas'), new Date(2003, 1, 1, 0, 0)),
      new SearchResult('monday3', 'img_1234.png', 'image/png', Location.parse('Paris, France'), new Date(2002, 1, 1, 0, 0)),
      new SearchResult('monday4', 'img_1234.jpg', 'image/jpeg', Location.parse('Nice, France'), new Date(2000, 1, 1, 0, 0)),
      new SearchResult('monday5', 'img_1234.mov', 'video/mp4', Location.parse('Hilo, Hawaii'), new Date(2004, 1, 1, 0, 0)),
    ];
    const mockRecordRepository = recordRepositoryMock({
      queryByTags: mock(() => Promise.resolve(results)),
    });
    const usecase = SearchAssets({ recordRepository: mockRecordRepository });
    // act
    const actual = await usecase(new SearchParams().addTag('cat').setSortField(SortField.Date).setSortOrder(SortOrder.Descending));
    // assert
    expect(actual).toHaveLength(5);
    expect(actual[0]?.assetId).toEqual('monday5');
    expect(actual[1]?.assetId).toEqual('monday2');
    expect(actual[2]?.assetId).toEqual('monday3');
    expect(actual[3]?.assetId).toEqual('monday1');
    expect(actual[4]?.assetId).toEqual('monday4');
  });

  test('should sort results by ascending identifier', async function () {
    // arrange
    const results = [
      new SearchResult('monday3', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2001, 1, 1, 0, 0)),
      new SearchResult('monday2', 'img_1234.mov', 'video/mp4', Location.parse('Paris, Texas'), new Date(2003, 1, 1, 0, 0)),
      new SearchResult('monday1', 'img_1234.png', 'image/png', Location.parse('Paris, France'), new Date(2002, 1, 1, 0, 0)),
      new SearchResult('monday5', 'img_1234.jpg', 'image/jpeg', Location.parse('Nice, France'), new Date(2000, 1, 1, 0, 0)),
      new SearchResult('monday4', 'img_1234.mov', 'video/mp4', Location.parse('Hilo, Hawaii'), new Date(2004, 1, 1, 0, 0)),
    ];
    const mockRecordRepository = recordRepositoryMock({
      queryByTags: mock(() => Promise.resolve(results)),
    });
    const usecase = SearchAssets({ recordRepository: mockRecordRepository });
    // act
    const actual = await usecase(new SearchParams().addTag('cat').setSortField(SortField.Identifier));
    // assert
    expect(actual).toHaveLength(5);
    expect(actual[0]?.assetId).toEqual('monday1');
    expect(actual[1]?.assetId).toEqual('monday2');
    expect(actual[2]?.assetId).toEqual('monday3');
    expect(actual[3]?.assetId).toEqual('monday4');
    expect(actual[4]?.assetId).toEqual('monday5');
  });

  test('should sort results by descending identifier', async function () {
    // arrange
    const results = [
      new SearchResult('monday3', 'img_1234.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2001, 1, 1, 0, 0)),
      new SearchResult('monday2', 'img_1234.mov', 'video/mp4', Location.parse('Paris, Texas'), new Date(2003, 1, 1, 0, 0)),
      new SearchResult('monday1', 'img_1234.png', 'image/png', Location.parse('Paris, France'), new Date(2002, 1, 1, 0, 0)),
      new SearchResult('monday5', 'img_1234.jpg', 'image/jpeg', Location.parse('Nice, France'), new Date(2000, 1, 1, 0, 0)),
      new SearchResult('monday4', 'img_1234.mov', 'video/mp4', Location.parse('Hilo, Hawaii'), new Date(2004, 1, 1, 0, 0)),
    ];
    const mockRecordRepository = recordRepositoryMock({
      queryByTags: mock(() => Promise.resolve(results)),
    });
    const usecase = SearchAssets({ recordRepository: mockRecordRepository });
    // act
    const actual = await usecase(new SearchParams().addTag('cat').setSortField(SortField.Identifier).setSortOrder(SortOrder.Descending));
    // assert
    expect(actual).toHaveLength(5);
    expect(actual[0]?.assetId).toEqual('monday5');
    expect(actual[1]?.assetId).toEqual('monday4');
    expect(actual[2]?.assetId).toEqual('monday3');
    expect(actual[3]?.assetId).toEqual('monday2');
    expect(actual[4]?.assetId).toEqual('monday1');
  });

  test('should sort results by ascending filename', async function () {
    // arrange
    const results = [
      new SearchResult('monday1', 'img_2345.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2001, 1, 1, 0, 0)),
      new SearchResult('monday2', 'img_1234.mov', 'video/mp4', Location.parse('Paris, Texas'), new Date(2003, 1, 1, 0, 0)),
      new SearchResult('monday3', 'img_3456.png', 'image/png', Location.parse('Paris, France'), new Date(2002, 1, 1, 0, 0)),
      new SearchResult('monday4', 'img_5678.jpg', 'image/jpeg', Location.parse('Nice, France'), new Date(2000, 1, 1, 0, 0)),
      new SearchResult('monday5', 'img_4567.mov', 'video/mp4', Location.parse('Hilo, Hawaii'), new Date(2004, 1, 1, 0, 0)),
    ];
    const mockRecordRepository = recordRepositoryMock({
      queryByTags: mock(() => Promise.resolve(results)),
    });
    const usecase = SearchAssets({ recordRepository: mockRecordRepository });
    // act
    const actual = await usecase(new SearchParams().addTag('cat').setSortField(SortField.Filename));
    // assert
    expect(actual).toHaveLength(5);
    expect(actual[0]?.assetId).toEqual('monday2');
    expect(actual[1]?.assetId).toEqual('monday1');
    expect(actual[2]?.assetId).toEqual('monday3');
    expect(actual[3]?.assetId).toEqual('monday5');
    expect(actual[4]?.assetId).toEqual('monday4');
  });

  test('should sort results by descending filename', async function () {
    // arrange
    const results = [
      new SearchResult('monday1', 'img_2345.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2001, 1, 1, 0, 0)),
      new SearchResult('monday2', 'img_1234.mov', 'video/mp4', Location.parse('Paris, Texas'), new Date(2003, 1, 1, 0, 0)),
      new SearchResult('monday3', 'img_3456.png', 'image/png', Location.parse('Paris, France'), new Date(2002, 1, 1, 0, 0)),
      new SearchResult('monday4', 'img_5678.jpg', 'image/jpeg', Location.parse('Nice, France'), new Date(2000, 1, 1, 0, 0)),
      new SearchResult('monday5', 'img_4567.mov', 'video/mp4', Location.parse('Hilo, Hawaii'), new Date(2004, 1, 1, 0, 0)),
    ];
    const mockRecordRepository = recordRepositoryMock({
      queryByTags: mock(() => Promise.resolve(results)),
    });
    const usecase = SearchAssets({ recordRepository: mockRecordRepository });
    // act
    const actual = await usecase(new SearchParams().addTag('cat').setSortField(SortField.Filename).setSortOrder(SortOrder.Descending));
    // assert
    expect(actual).toHaveLength(5);
    expect(actual[0]?.assetId).toEqual('monday4');
    expect(actual[1]?.assetId).toEqual('monday5');
    expect(actual[2]?.assetId).toEqual('monday3');
    expect(actual[3]?.assetId).toEqual('monday1');
    expect(actual[4]?.assetId).toEqual('monday2');
  });

  test('should sort results by ascending media type', async function () {
    // arrange
    const results = [
      new SearchResult('monday1', 'img_2345.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2001, 1, 1, 0, 0)),
      new SearchResult('monday2', 'img_1234.mov', 'video/mp4', Location.parse('Paris, Texas'), new Date(2003, 1, 1, 0, 0)),
      new SearchResult('monday3', 'img_3456.png', 'image/png', Location.parse('Paris, France'), new Date(2002, 1, 1, 0, 0)),
      new SearchResult('monday4', 'secrets.txt', 'text/plain', Location.parse('home'), new Date(2000, 1, 1, 0, 0)),
      new SearchResult('monday5', 'img_4567.mov', 'video/mov', Location.parse('Hilo, Hawaii'), new Date(2004, 1, 1, 0, 0)),
    ];
    const mockRecordRepository = recordRepositoryMock({
      queryByTags: mock(() => Promise.resolve(results)),
    });
    const usecase = SearchAssets({ recordRepository: mockRecordRepository });
    // act
    const actual = await usecase(new SearchParams().addTag('cat').setSortField(SortField.MediaType));
    // assert
    expect(actual).toHaveLength(5);
    expect(actual[0]?.assetId).toEqual('monday1');
    expect(actual[1]?.assetId).toEqual('monday3');
    expect(actual[2]?.assetId).toEqual('monday4');
    expect(actual[3]?.assetId).toEqual('monday5');
    expect(actual[4]?.assetId).toEqual('monday2');
  });

  test('should sort results by descending media type', async function () {
    // arrange
    const results = [
      new SearchResult('monday1', 'img_2345.jpg', 'image/jpeg', Location.parse('Oahu, Hawaii'), new Date(2001, 1, 1, 0, 0)),
      new SearchResult('monday2', 'img_1234.mov', 'video/mp4', Location.parse('Paris, Texas'), new Date(2003, 1, 1, 0, 0)),
      new SearchResult('monday3', 'img_3456.png', 'image/png', Location.parse('Paris, France'), new Date(2002, 1, 1, 0, 0)),
      new SearchResult('monday4', 'secrets.txt', 'text/plain', Location.parse('home'), new Date(2000, 1, 1, 0, 0)),
      new SearchResult('monday5', 'img_4567.mov', 'video/mov', Location.parse('Hilo, Hawaii'), new Date(2004, 1, 1, 0, 0)),
    ];
    const mockRecordRepository = recordRepositoryMock({
      queryByTags: mock(() => Promise.resolve(results)),
    });
    const usecase = SearchAssets({ recordRepository: mockRecordRepository });
    // act
    const actual = await usecase(new SearchParams().addTag('cat').setSortField(SortField.MediaType).setSortOrder(SortOrder.Descending));
    // assert
    expect(actual).toHaveLength(5);
    expect(actual[0]?.assetId).toEqual('monday2');
    expect(actual[1]?.assetId).toEqual('monday5');
    expect(actual[2]?.assetId).toEqual('monday4');
    expect(actual[3]?.assetId).toEqual('monday3');
    expect(actual[4]?.assetId).toEqual('monday1');
  });
});
