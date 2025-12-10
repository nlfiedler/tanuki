//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, mock, test } from 'bun:test';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { Geocoded } from 'tanuki/server/domain/entities/location.ts';
import ImportAsset from 'tanuki/server/domain/usecases/import-asset.ts';
import {
  blobRepositoryMock,
  locationRepositoryMock,
  recordRepositoryMock
} from './mocking.ts';

describe('ImportAsset use case', function () {
  test('should import a new asset', async function () {
    // arrange
    const mockRecordRepository = recordRepositoryMock({});
    const mockBlobRepository = blobRepositoryMock({});
    const mockLocationRepository = locationRepositoryMock({});
    const filepath = './test/fixtures/dcp_1069.jpg';
    const mimetype = 'image/jpeg';
    const modified = new Date();
    const usecase = ImportAsset({
      recordRepository: mockRecordRepository,
      blobRepository: mockBlobRepository,
      locationRepository: mockLocationRepository
    });
    // act
    const actual = await usecase(filepath, 'dcp_1069.jpg', mimetype, modified);
    // assert
    expect(actual.checksum).toEqual(
      'sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07'
    );
    expect(actual.byteLength).toEqual(80_977);
    expect(actual.mediaType).toEqual('image/jpeg');
    expect(actual.originalDate?.getFullYear()).toEqual(2003);
    expect(actual.originalDate?.getMonth()).toEqual(8); // zero-based
    expect(actual.originalDate?.getDate()).toEqual(3);
    expect(mockRecordRepository.getAssetByDigest).toHaveBeenCalledTimes(1);
    expect(mockBlobRepository.storeBlob).toHaveBeenCalledTimes(1);
    expect(mockRecordRepository.putAsset).toHaveBeenCalledTimes(1);
    mock.clearAllMocks();
  });

  test('should ignore importing an existing asset', async function () {
    // arrange
    const asset = new Asset('abc123');
    asset.checksum =
      'sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07';
    asset.byteLength = 80_977;
    asset.mediaType = 'image/jpeg';
    const mockRecordRepository = recordRepositoryMock({
      getAssetByDigest: mock(() => Promise.resolve(asset))
    });
    const mockBlobRepository = blobRepositoryMock({});
    const mockLocationRepository = locationRepositoryMock({});
    const filepath = './test/fixtures/dcp_1069.jpg';
    const mimetype = 'image/jpeg';
    const modified = new Date();
    const usecase = ImportAsset({
      recordRepository: mockRecordRepository,
      blobRepository: mockBlobRepository,
      locationRepository: mockLocationRepository
    });
    // act
    const actual = await usecase(filepath, 'dcp_1069.jpg', mimetype, modified);
    // assert
    expect(actual.checksum).toEqual(
      'sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07'
    );
    expect(actual.byteLength).toEqual(80_977);
    expect(actual.mediaType).toEqual('image/jpeg');
    expect(mockRecordRepository.getAssetByDigest).toHaveBeenCalledTimes(1);
    expect(mockBlobRepository.storeBlob).toHaveBeenCalledTimes(1);
    expect(mockRecordRepository.putAsset).toHaveBeenCalledTimes(0);
    mock.clearAllMocks();
  });

  test('should import an asset with GPS coordinates', async function () {
    // arrange
    const mockRecordRepository = recordRepositoryMock({});
    const mockBlobRepository = blobRepositoryMock({});
    const mockLocationRepository = locationRepositoryMock({
      findLocation: mock(() =>
        Promise.resolve(new Geocoded('Portland', 'Oregon', 'USA'))
      )
    });
    // file needs GPS coordinates for the location repository to be invoked
    const filepath = './test/fixtures/IMG_0385.JPG';
    const mimetype = 'image/jpeg';
    const modified = new Date();
    const usecase = ImportAsset({
      recordRepository: mockRecordRepository,
      blobRepository: mockBlobRepository,
      locationRepository: mockLocationRepository
    });
    // act
    const actual = await usecase(filepath, 'IMG_0385.JPG', mimetype, modified);
    // assert
    expect(actual.checksum).toEqual(
      'sha256-d020066fd41970c2eebc51b1e712a500de4966cef0daf4890dc238d80cbaebb2'
    );
    expect(actual.byteLength).toEqual(59_908);
    expect(actual.mediaType).toEqual('image/jpeg');
    expect(actual.originalDate?.getFullYear()).toEqual(2024);
    expect(actual.originalDate?.getMonth()).toEqual(1); // zero-based
    expect(actual.originalDate?.getDate()).toEqual(9);
    expect(actual.location?.city).toEqual('Portland');
    expect(actual.location?.region).toEqual('Oregon');
    expect(mockRecordRepository.getAssetByDigest).toHaveBeenCalledTimes(1);
    expect(mockBlobRepository.storeBlob).toHaveBeenCalledTimes(1);
    expect(mockRecordRepository.putAsset).toHaveBeenCalledTimes(1);
    expect(mockLocationRepository.findLocation).toHaveBeenCalledTimes(1);
    mock.clearAllMocks();
  });
});
