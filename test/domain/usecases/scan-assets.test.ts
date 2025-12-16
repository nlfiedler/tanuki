//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, mock, test } from 'bun:test';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { Location } from 'tanuki/server/domain/entities/location.ts';
import ScanAssets from 'tanuki/server/domain/usecases/scan-assets.ts';
import { recordRepositoryMock } from './mocking.ts';

describe('ScanAssets use case', function () {
  test('should do nothing when empty query', async function () {
    // arrange
    const mockRecordRepository = recordRepositoryMock({});
    const usecase = ScanAssets({ recordRepository: mockRecordRepository });
    // act
    const actual = await usecase('');
    // assert
    expect(actual).toHaveLength(0);
    expect(mockRecordRepository.fetchAssets).toHaveBeenCalledTimes(0);
    mock.clearAllMocks();
  });

  test('should find nothing when zero assets', async function () {
    // arrange
    const mockRecordRepository = recordRepositoryMock({});
    const usecase = ScanAssets({ recordRepository: mockRecordRepository });
    // act
    const actual = await usecase('tag:kitten');
    // assert
    expect(actual).toHaveLength(0);
    expect(mockRecordRepository.fetchAssets).toHaveBeenCalledTimes(1);
    mock.clearAllMocks();
  });

  test('should find nothing for non-matching query', async function () {
    // arrange
    const assets = [
      new Asset('monday1')
        .setChecksum('sha1-cafebabe')
        .setFilename('img_1234.jpg')
        .setByteLength(1024)
        .setMediaType('image/jpeg')
        .setTags(['cat', 'dog'])
        .setImportDate(new Date(2018, 4, 31, 21, 10, 11))
        .setCaption('#cat and #dog @hawaii')
        .setLocation(Location.parse('Oahu, Hawaii'))
    ];
    const mockFn = mock();
    mockFn.mockImplementationOnce(() => Promise.resolve([assets, 'done']));
    mockFn.mockImplementation(() => Promise.resolve([[], 'done']));
    const mockRecordRepository = recordRepositoryMock({
      fetchAssets: mockFn
    });
    const usecase = ScanAssets({ recordRepository: mockRecordRepository });
    // act
    const actual = await usecase('tag:kitten');
    // assert
    expect(actual).toHaveLength(0);
    expect(mockRecordRepository.fetchAssets).toHaveBeenCalledTimes(2);
    mock.clearAllMocks();
  });

  test('should find all matching assets', async function () {
    // arrange
    const assets1 = [
      new Asset('monday1')
        .setChecksum('sha1-cafebabe')
        .setFilename('img_1234.jpg')
        .setByteLength(1024)
        .setMediaType('image/jpeg')
        .setTags(['cat', 'dog'])
        .setImportDate(new Date(2018, 4, 31, 21, 10, 11))
        .setCaption('#cat and #dog @hawaii')
        .setLocation(Location.parse('Oahu, Hawaii')),
      new Asset('monday2')
        .setChecksum('sha1-cafed00d')
        .setFilename('img_1001.jpg')
        .setByteLength(2048)
        .setMediaType('image/jpeg')
        .setTags(['cat', 'mouse'])
        .setImportDate(new Date(2015, 8, 1, 21, 10, 11))
        .setCaption('playing in the sun')
        .setLocation(Location.parse('beach'))
    ];
    const assets2 = [
      new Asset('tuesday2')
        .setChecksum('sha1-babecafe')
        .setFilename('img_2345.jpg')
        .setByteLength(1024)
        .setMediaType('image/jpeg')
        .setTags(['bird', 'dog'])
        .setImportDate(new Date(2003, 7, 30, 21, 10, 11))
        .setCaption('#bird and #dog outside')
        .setLocation(Location.parse('Paris, Texas'))
    ];
    const mockFn = mock();
    mockFn.mockImplementationOnce(() => Promise.resolve([assets1, 'yay']));
    mockFn.mockImplementationOnce(() => Promise.resolve([assets2, 'done']));
    mockFn.mockImplementation(() => Promise.resolve([[], 'done']));
    const mockRecordRepository = recordRepositoryMock({
      fetchAssets: mockFn
    });
    const usecase = ScanAssets({ recordRepository: mockRecordRepository });
    // act
    const actual = await usecase('tag:cat');
    // assert
    expect(actual).toHaveLength(2);
    expect(actual[0]?.assetId).toEqual('monday1');
    expect(actual[1]?.assetId).toEqual('monday2');
    expect(mockRecordRepository.fetchAssets).toHaveBeenCalledTimes(3);
    mock.clearAllMocks();
  });
});
