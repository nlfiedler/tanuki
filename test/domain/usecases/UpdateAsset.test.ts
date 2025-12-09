//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, mock, test } from 'bun:test';
import { Asset } from 'tanuki/server/domain/entities/Asset.ts';
import { AssetInput } from 'tanuki/server/domain/entities/AssetInput.ts';
import { Location } from 'tanuki/server/domain/entities/Location.ts';
import UpdateAsset from 'tanuki/server/domain/usecases/UpdateAsset.ts';
import { recordRepositoryMock } from './mocking.ts';

describe('UpdateAsset use case', function () {
  test('should do nothing with empty inputs', async function () {
    // arrange
    const asset = new Asset('kittens1');
    asset.checksum =
      'sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07';
    asset.byteLength = 80977;
    asset.mediaType = 'image/jpeg';
    asset.tags = ['kittens', 'playing'];
    const mockRecordRepository = recordRepositoryMock({
      getAssetById: mock((_assetId: string) => Promise.resolve(asset))
    });
    const usecase = UpdateAsset({
      recordRepository: mockRecordRepository
    });
    // act
    const updated = await usecase(new AssetInput('kittens1'));
    // assert
    expect(updated.checksum).toEqual(
      'sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07'
    );
    expect(updated.byteLength).toEqual(80977);
    expect(updated.mediaType).toEqual('image/jpeg');
    expect(updated.tags).toHaveLength(2);
    expect(updated.tags[0]).toEqual('kittens');
    expect(updated.tags[1]).toEqual('playing');
    expect(mockRecordRepository.getAssetById).toHaveBeenCalledTimes(1);
    expect(mockRecordRepository.putAsset).toHaveBeenCalledTimes(1);
    mock.clearAllMocks();
  });

  test('should wipe out tags if input has empty list', async function () {
    // arrange
    const asset = new Asset('kittens1');
    asset.checksum =
      'sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07';
    asset.byteLength = 80977;
    asset.mediaType = 'image/jpeg';
    asset.tags = ['kittens', 'playing'];
    const mockRecordRepository = recordRepositoryMock({
      getAssetById: mock((_assetId: string) => Promise.resolve(asset))
    });
    const usecase = UpdateAsset({
      recordRepository: mockRecordRepository
    });
    // act
    const updated = await usecase(new AssetInput('kittens1').setTags([]));
    // assert
    expect(updated.tags).toHaveLength(0);
    expect(mockRecordRepository.getAssetById).toHaveBeenCalledTimes(1);
    expect(mockRecordRepository.putAsset).toHaveBeenCalledTimes(1);
    mock.clearAllMocks();
  });

  test('should merge duplicate tags', async function () {
    // arrange
    const asset = new Asset('kittens1');
    asset.checksum =
      'sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07';
    asset.byteLength = 80977;
    asset.mediaType = 'image/jpeg';
    asset.tags = ['kittens', 'playing'];
    const mockRecordRepository = recordRepositoryMock({
      getAssetById: mock((_assetId: string) => Promise.resolve(asset))
    });
    const usecase = UpdateAsset({
      recordRepository: mockRecordRepository
    });
    // act
    const updated = await usecase(
      new AssetInput('kittens1').setTags([
        'kittens',
        'kittens',
        'kittens',
        'playing'
      ])
    );
    // assert
    expect(updated.tags).toHaveLength(2);
    expect(updated.tags[0]).toEqual('kittens');
    expect(updated.tags[1]).toEqual('playing');
    expect(mockRecordRepository.getAssetById).toHaveBeenCalledTimes(1);
    expect(mockRecordRepository.putAsset).toHaveBeenCalledTimes(1);
    mock.clearAllMocks();
  });

  test('should replace filename and media type if given', async function () {
    // arrange
    const asset = new Asset('kittens1');
    asset.checksum =
      'sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07';
    asset.byteLength = 80977;
    asset.mediaType = 'image/jpeg';
    asset.filename = 'IMG_1234.JPG';
    asset.tags = ['kittens', 'playing'];
    const mockRecordRepository = recordRepositoryMock({
      getAssetById: mock((_assetId: string) => Promise.resolve(asset))
    });
    const usecase = UpdateAsset({
      recordRepository: mockRecordRepository
    });
    // act
    const updated = await usecase(
      new AssetInput('kittens1')
        .setFilename('img_2345.png')
        .setMediaType('image/png')
    );
    // assert
    expect(updated).toEqual(asset);
    expect(updated.mediaType).toEqual('image/png');
    expect(updated.filename).toEqual('img_2345.png');
    expect(mockRecordRepository.getAssetById).toHaveBeenCalledTimes(1);
    expect(mockRecordRepository.putAsset).toHaveBeenCalledTimes(1);
    mock.clearAllMocks();
  });

  test('should not overwrite with blank filename, media type', async function () {
    // arrange
    const asset = new Asset('kittens1');
    asset.checksum =
      'sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07';
    asset.byteLength = 80977;
    asset.mediaType = 'image/jpeg';
    asset.filename = 'IMG_1234.JPG';
    asset.tags = ['kittens', 'playing'];
    const mockRecordRepository = recordRepositoryMock({
      getAssetById: mock((_assetId: string) => Promise.resolve(asset))
    });
    const usecase = UpdateAsset({
      recordRepository: mockRecordRepository
    });
    // act
    const updated = await usecase(
      new AssetInput('kittens1').setFilename('').setMediaType('')
    );
    // assert
    expect(updated).toEqual(asset);
    expect(updated.mediaType).toEqual('image/jpeg');
    expect(updated.filename).toEqual('IMG_1234.JPG');
    expect(mockRecordRepository.getAssetById).toHaveBeenCalledTimes(1);
    expect(mockRecordRepository.putAsset).toHaveBeenCalledTimes(1);
    mock.clearAllMocks();
  });

  test('should replace tags with new values', async function () {
    // arrange
    const asset = new Asset('kittens1');
    asset.checksum =
      'sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07';
    asset.byteLength = 80977;
    asset.mediaType = 'image/jpeg';
    asset.tags = ['kittens', 'playing'];
    const mockRecordRepository = recordRepositoryMock({
      getAssetById: mock((_assetId: string) => Promise.resolve(asset))
    });
    const usecase = UpdateAsset({
      recordRepository: mockRecordRepository
    });
    // act
    const updated = await usecase(new AssetInput('kittens1').addTag('furry'));
    // assert
    expect(updated.tags).toHaveLength(1);
    expect(updated.tags[0]).toEqual('furry');
    expect(mockRecordRepository.getAssetById).toHaveBeenCalledTimes(1);
    expect(mockRecordRepository.putAsset).toHaveBeenCalledTimes(1);
    mock.clearAllMocks();
  });

  test('should replace tags and merge with caption', async function () {
    // arrange
    const asset = new Asset('kittens1');
    asset.checksum =
      'sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07';
    asset.byteLength = 80977;
    asset.mediaType = 'image/jpeg';
    asset.tags = ['cute'];
    const mockRecordRepository = recordRepositoryMock({
      getAssetById: mock((_assetId: string) => Promise.resolve(asset))
    });
    const usecase = UpdateAsset({
      recordRepository: mockRecordRepository
    });
    // act
    const updated = await usecase(
      new AssetInput('kittens1')
        .addTag('puppies')
        .setCaption('#kittens fighting #kittens')
    );
    // assert
    expect(updated.tags).toHaveLength(2);
    expect(updated.tags[0]).toEqual('kittens');
    expect(updated.tags[1]).toEqual('puppies');
    expect(updated.caption).toEqual('#kittens fighting #kittens');
    expect(mockRecordRepository.getAssetById).toHaveBeenCalledTimes(1);
    expect(mockRecordRepository.putAsset).toHaveBeenCalledTimes(1);
    mock.clearAllMocks();
  });

  test('should update preferred date-time', async function () {
    // arrange
    const asset = new Asset('kittens1');
    asset.checksum =
      'sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07';
    asset.byteLength = 80977;
    asset.mediaType = 'image/jpeg';
    asset.tags = ['cute'];
    const mockRecordRepository = recordRepositoryMock({
      getAssetById: mock((_assetId: string) => Promise.resolve(asset))
    });
    const usecase = UpdateAsset({
      recordRepository: mockRecordRepository
    });
    // act
    const updated = await usecase(
      new AssetInput('kittens1').setDatetime(new Date(1996, 1, 1))
    );
    // assert
    expect(updated.bestDate()).toEqual(new Date(1996, 1, 1));
    expect(mockRecordRepository.getAssetById).toHaveBeenCalledTimes(1);
    expect(mockRecordRepository.putAsset).toHaveBeenCalledTimes(1);
    mock.clearAllMocks();
  });

  test('should clear location if explicitly provided', async function () {
    // arrange
    const asset = new Asset('kittens1');
    asset.checksum =
      'sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07';
    asset.byteLength = 80977;
    asset.mediaType = 'image/jpeg';
    asset.location = Location.parse('mini town; Pleasanton, CA');
    asset.tags = ['kittens', 'playing'];
    const mockRecordRepository = recordRepositoryMock({
      getAssetById: mock((_assetId: string) => Promise.resolve(asset))
    });
    const usecase = UpdateAsset({
      recordRepository: mockRecordRepository
    });
    // act
    const inputLocation = Location.parse('Pleasanton, CA');
    inputLocation.label = '';
    const updated = await usecase(
      new AssetInput('kittens1').setLocation(inputLocation)
    );
    // assert
    expect(updated.location).toEqual(Location.parse('Pleasanton, CA'));
    expect(mockRecordRepository.getAssetById).toHaveBeenCalledTimes(1);
    expect(mockRecordRepository.putAsset).toHaveBeenCalledTimes(1);
    mock.clearAllMocks();
  });

  test('should not clobber location, merge tags', async function () {
    // arrange
    const asset = new Asset('kittens1');
    asset.checksum =
      'sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07';
    asset.byteLength = 80977;
    asset.mediaType = 'image/jpeg';
    asset.location = Location.parse('mini town; Pleasanton, CA');
    asset.tags = ['kittens', 'playing'];
    const mockRecordRepository = recordRepositoryMock({
      getAssetById: mock((_assetId: string) => Promise.resolve(asset))
    });
    const usecase = UpdateAsset({
      recordRepository: mockRecordRepository
    });
    // act
    const updated = await usecase(
      new AssetInput('kittens1').setCaption(
        '#kittens and #puppies #playing @beach'
      )
    );
    // assert
    expect(updated.tags).toHaveLength(3);
    expect(updated.tags[0]).toEqual('kittens');
    expect(updated.tags[1]).toEqual('playing');
    expect(updated.tags[2]).toEqual('puppies');
    expect(updated.location).toEqual(
      Location.parse('mini town; Pleasanton, CA')
    );
    expect(updated.caption).toEqual('#kittens and #puppies #playing @beach');
    expect(mockRecordRepository.getAssetById).toHaveBeenCalledTimes(1);
    expect(mockRecordRepository.putAsset).toHaveBeenCalledTimes(1);
    mock.clearAllMocks();
  });
});
