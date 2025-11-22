//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, mock, test } from "bun:test";
import { Asset } from 'tanuki/server/domain/entities/Asset.ts';
import ImportAsset from 'tanuki/server/domain/usecases/ImportAsset.ts';
import { blobRepositoryMock, recordRepositoryMock } from './mocking.ts';

describe('ImportAsset use case', function () {
  test('should import a new asset', async function () {
    // arrange
    const mockRecordRepository = recordRepositoryMock({});
    const mockBlobRepository = blobRepositoryMock({});
    const filepath = './test/fixtures/dcp_1069.jpg';
    const mimetype = 'image/jpeg';
    const modified = new Date();
    const usecase = ImportAsset({ recordRepository: mockRecordRepository, blobRepository: mockBlobRepository });
    // act
    const actual = await usecase(filepath, 'dcp_1069.jpg', mimetype, modified);
    // assert
    expect(actual.checksum).toEqual('sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07');
    expect(actual.byteLength).toEqual(80977);
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
    asset.checksum = 'sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07';
    asset.byteLength = 80977;
    asset.mediaType = 'image/jpeg';
    const mockRecordRepository = recordRepositoryMock({
      getAssetByDigest: mock(() => Promise.resolve(asset))
    });
    const mockBlobRepository = blobRepositoryMock({});
    const filepath = './test/fixtures/dcp_1069.jpg';
    const mimetype = 'image/jpeg';
    const modified = new Date();
    const usecase = ImportAsset({ recordRepository: mockRecordRepository, blobRepository: mockBlobRepository });
    // act
    const actual = await usecase(filepath, 'dcp_1069.jpg', mimetype, modified);
    // assert
    expect(actual.checksum).toEqual('sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07');
    expect(actual.byteLength).toEqual(80977);
    expect(actual.mediaType).toEqual('image/jpeg');
    expect(mockRecordRepository.getAssetByDigest).toHaveBeenCalledTimes(1);
    expect(mockBlobRepository.storeBlob).toHaveBeenCalledTimes(1);
    expect(mockRecordRepository.putAsset).toHaveBeenCalledTimes(0);
    mock.clearAllMocks();
  });

  // TODO: test with call to reverse geocoder
});
