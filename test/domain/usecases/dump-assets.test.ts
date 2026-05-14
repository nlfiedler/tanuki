//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, mock, test } from 'bun:test';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { AssetMetadata } from 'tanuki/server/domain/entities/asset-metadata.ts';
import { Location } from 'tanuki/server/domain/entities/location.ts';
import DumpAssets from 'tanuki/server/domain/usecases/dump-assets.ts';
import { recordRepositoryMock } from './mocking.ts';

describe('DumpAssets use case', function () {
  test('should return nothing for empty database', async function () {
    // arrange
    const mockRecordRepository = recordRepositoryMock({
      fetchAssets: mock(() =>
        Promise.resolve([[], 'done'] as [Array<Asset>, any])
      )
    });
    const usecase = DumpAssets({
      recordRepository: mockRecordRepository
    });
    // act
    const results = [];
    for await (const entry of usecase(10)) {
      results.push(entry);
    }
    // assert
    expect(results).toHaveLength(0);
    expect(mockRecordRepository.fetchAssets).toHaveBeenCalledTimes(1);
    mock.clearAllMocks();
  });

  test('should return the one record in the database', async function () {
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
    const usecase = DumpAssets({
      recordRepository: mockRecordRepository
    });
    // act
    const results = [];
    for await (const entry of usecase(10)) {
      results.push(entry);
    }
    // assert
    expect(results).toHaveLength(1);
    expect(results[0]?.key).toEqual('monday1');
    expect(results[0]?.checksum).toEqual('sha1-cafebabe');
    expect(results[0]?.filename).toEqual('img_1234.jpg');
    expect(results[0]?.byte_length).toEqual(1024);
    expect(results[0]?.media_type).toEqual('image/jpeg');
    expect(results[0]?.tags).toEqual(['cat', 'dog']);
    expect(results[0]?.import_date.getFullYear()).toEqual(2018);
    expect(results[0]?.caption).toEqual('#cat and #dog @hawaii');
    expect(results[0]?.location).toEqual({ l: null, c: 'Oahu', r: 'Hawaii' });
    expect(results[0]?.user_date).toBeNull();
    expect(results[0]?.original_date).toBeNull();
    expect(results[0]?.dimensions).toBeNull();
    expect(results[0]?.metadata).toBeNull();
    expect(mockRecordRepository.fetchAssets).toHaveBeenCalledTimes(2);
    mock.clearAllMocks();
  });

  test('should serialize metadata when present', async function () {
    // arrange
    const metadata = new AssetMetadata();
    metadata.cameraMake = 'Canon';
    metadata.cameraModel = 'EOS 5D';
    metadata.fNumber = 2.8;
    metadata.iso = 400;
    metadata.displayWidth = 4000;
    metadata.displayHeight = 3000;
    metadata.raw = { Make: 'Canon', Model: 'EOS 5D' };
    const assets = [
      new Asset('monday1')
        .setChecksum('sha1-cafebabe')
        .setFilename('img_1234.jpg')
        .setByteLength(1024)
        .setMediaType('image/jpeg')
        .setTags(['cat'])
        .setImportDate(new Date(2018, 4, 31, 21, 10, 11))
        .setMetadata(metadata)
    ];
    const mockFn = mock();
    mockFn.mockImplementationOnce(() => Promise.resolve([assets, 'done']));
    mockFn.mockImplementation(() => Promise.resolve([[], 'done']));
    const mockRecordRepository = recordRepositoryMock({
      fetchAssets: mockFn
    });
    const usecase = DumpAssets({
      recordRepository: mockRecordRepository
    });
    // act
    const results = [];
    for await (const entry of usecase(10)) {
      results.push(entry);
    }
    // assert
    expect(results).toHaveLength(1);
    const dumped: any = results[0]?.metadata;
    expect(dumped).not.toBeNull();
    expect(dumped.cameraMake).toEqual('Canon');
    expect(dumped.cameraModel).toEqual('EOS 5D');
    expect(dumped.fNumber).toEqual(2.8);
    expect(dumped.iso).toEqual(400);
    expect(dumped.displayWidth).toEqual(4000);
    expect(dumped.displayHeight).toEqual(3000);
    expect(dumped.raw).toEqual({ Make: 'Canon', Model: 'EOS 5D' });
    mock.clearAllMocks();
  });

  test('should loop mulitple times to get all records', async function () {
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
    const usecase = DumpAssets({
      recordRepository: mockRecordRepository
    });
    // act
    const results = [];
    for await (const entry of usecase(10)) {
      results.push(entry);
    }
    // assert
    expect(results).toHaveLength(3);
    expect(results[0]?.key).toEqual('monday1');
    expect(results[1]?.key).toEqual('monday2');
    const location: unknown = results[1]?.location;
    expect(location as string).toEqual('beach');
    expect(results[2]?.key).toEqual('tuesday2');
    expect(mockRecordRepository.fetchAssets).toHaveBeenCalledTimes(3);
    mock.clearAllMocks();
  });
});
