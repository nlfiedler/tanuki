//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, mock, test } from 'bun:test';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { Location } from 'tanuki/server/domain/entities/location.ts';
import ReplaceAsset from 'tanuki/server/domain/usecases/replace-asset.ts';
import { blobRepositoryMock, recordRepositoryMock } from './mocking.ts';

describe('ReplaceAsset use case', function () {
  test('should merge old values into new and remove old assets', async function () {
    // arrange
    const oldAsset = new Asset('kittens1');
    oldAsset.checksum = 'sha1-7e8b23ae02e9dce31c8094fec63097e958749f9d';
    oldAsset.byteLength = 80_977;
    oldAsset.mediaType = 'image/jpeg';
    oldAsset.tags = ['kittens', 'playing'];
    oldAsset.caption = 'kittens playing, yay!';
    oldAsset.location = Location.parse('Nice, France');

    const newAsset = new Asset('kitties2');
    newAsset.checksum = 'sha1-10846411651c5442be373f4d402c476ebcb3f644';
    newAsset.byteLength = 160_797;
    newAsset.mediaType = 'image/jpeg';

    const mockBlobRepository = blobRepositoryMock({});
    const mockRecordRepository = recordRepositoryMock({
      getAssetById: mock((assetId: string) => {
        if (assetId === 'kittens1') {
          return Promise.resolve(oldAsset);
        } else if (assetId === 'kitties2') {
          return Promise.resolve(newAsset);
        }
        return Promise.reject('wrong');
      })
    });
    const usecase = ReplaceAsset({
      blobRepository: mockBlobRepository,
      recordRepository: mockRecordRepository
    });
    // act
    const updated = await usecase('kittens1', 'kitties2');
    // assert
    expect(updated.key).toEqual('kitties2');
    expect(updated.checksum).toEqual('sha1-10846411651c5442be373f4d402c476ebcb3f644');
    expect(updated.byteLength).toEqual(160_797);
    expect(updated.tags).toHaveLength(2);
    expect(updated.tags[0]).toEqual('kittens');
    expect(updated.tags[1]).toEqual('playing');
    expect(updated.caption).toEqual('kittens playing, yay!');
    expect(updated.location).toEqual(Location.parse('Nice, France'));
    expect(mockRecordRepository.getAssetById).toHaveBeenCalledTimes(2);
    expect(mockRecordRepository.putAsset).toHaveBeenCalledTimes(1);
    expect(mockRecordRepository.deleteAsset).toHaveBeenCalledTimes(1);
    expect(mockBlobRepository.deleteBlob).toHaveBeenCalledTimes(1);
    mock.clearAllMocks();
  });
});
