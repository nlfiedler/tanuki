//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, mock, test } from 'bun:test';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import EditAssets from 'tanuki/server/domain/usecases/edit-assets.ts';
import * as ops from 'tanuki/server/domain/entities/edit.ts';
import { recordRepositoryMock } from './mocking.ts';

describe('EditAssets use case', function () {
  test('should do nothing when no inputs', async function () {
    // arrange
    const mockRecordRepository = recordRepositoryMock({});
    const usecase = EditAssets({
      recordRepository: mockRecordRepository
    });
    // act
    const mods = [new ops.TagAdd('dog')];
    const count = await usecase([], mods);
    // assert
    expect(count).toEqual(0);
    expect(mockRecordRepository.getAssetById).toHaveBeenCalledTimes(0);
    expect(mockRecordRepository.putAsset).toHaveBeenCalledTimes(0);
    mock.clearAllMocks();
  });

  test('should perform modifications on assets', async function () {
    // arrange
    const database = new Map<string, Asset>([
      ['monday1', new Asset('monday1').setTags(['cat', 'dog'])],
      ['tuesday2', new Asset('tuesday2').setTags(['cat', 'fluffy'])],
      ['wednesday3', new Asset('wednesday3').setTags(['fluffy', 'penguin'])],
      ['thursday4', new Asset('thursday4').setTags(['kitten'])]
    ]);
    const mockRecordRepository = recordRepositoryMock({
      getAssetById: mock((assetId: string) =>
        Promise.resolve(database.get(assetId) ?? null)
      ),
      putAsset: mock((asset: Asset) => {
        database.set(asset.key, asset);
        return Promise.resolve();
      })
    });
    const usecase = EditAssets({
      recordRepository: mockRecordRepository
    });
    // act
    const mods = [new ops.TagAdd('dog')];
    const assetIds = ['monday1', 'tuesday2', 'wednesday3', 'thursday4'];
    const count = await usecase(assetIds, mods);
    // assert
    expect(count).toEqual(3);
    expect(database.get('monday1')?.tags).toEqual(['cat', 'dog']);
    expect(database.get('tuesday2')?.tags).toEqual(['cat', 'fluffy', 'dog']);
    expect(database.get('wednesday3')?.tags).toEqual(['fluffy', 'penguin', 'dog']);
    expect(database.get('thursday4')?.tags).toEqual(['kitten', 'dog']);
    expect(mockRecordRepository.getAssetById).toHaveBeenCalledTimes(4);
    expect(mockRecordRepository.putAsset).toHaveBeenCalledTimes(3);
    mock.clearAllMocks();
  });
});
