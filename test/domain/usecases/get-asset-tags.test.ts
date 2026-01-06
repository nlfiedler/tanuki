//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, mock, test } from 'bun:test';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import GetAssetTags from 'tanuki/server/domain/usecases/get-asset-tags.ts';
import { recordRepositoryMock } from './mocking.ts';

describe('GetAssetTags use case', function () {
  test('should return empty list when given empty inputs', async function () {
    // arrange
    const mockRecordRepository = recordRepositoryMock({});
    const usecase = GetAssetTags({
      recordRepository: mockRecordRepository
    });
    // act
    const counts = await usecase([]);
    // assert
    expect(counts).toHaveLength(0);
    expect(mockRecordRepository.getAssetById).toHaveBeenCalledTimes(0);
    mock.clearAllMocks();
  });

  test('should return empty list when assets have no tags', async function () {
    // arrange
    const asset1 = new Asset('monday1').setChecksum('sha1-cafebabe');
    const asset2 = new Asset('tuesday2').setChecksum('sha1-babecafe');
    const mockFn = mock();
    mockFn.mockImplementationOnce(() => Promise.resolve(asset1));
    mockFn.mockImplementationOnce(() => Promise.resolve(asset2));
    const mockRecordRepository = recordRepositoryMock({
      getAssetById: mockFn
    });
    const usecase = GetAssetTags({
      recordRepository: mockRecordRepository
    });
    // act
    const counts = await usecase(['monday1', 'tuesday2']);
    // assert
    expect(counts).toHaveLength(0);
    expect(mockRecordRepository.getAssetById).toHaveBeenCalledTimes(2);
    mock.clearAllMocks();
  });

  test('should return valid counts when assets have tags', async function () {
    // arrange
    const asset1 = new Asset('monday1')
      .setChecksum('sha1-cafebabe')
      .setTags(['cat', 'dog']);
    const asset2 = new Asset('tuesday2')
      .setChecksum('sha1-babecafe')
      .setTags(['bird', 'cat']);
    const asset3 = new Asset('wednesday3')
      .setChecksum('sha1-cafed00d')
      .setTags(['cat', 'mouse']);
    const asset4 = new Asset('thursday4')
      .setChecksum('sha1-cafed00d')
      .setTags(['cat', 'dog']);
    const mockFn = mock();
    mockFn.mockImplementationOnce(() => Promise.resolve(asset1));
    mockFn.mockImplementationOnce(() => Promise.resolve(asset2));
    mockFn.mockImplementationOnce(() => Promise.resolve(asset3));
    mockFn.mockImplementationOnce(() => Promise.resolve(asset4));
    const mockRecordRepository = recordRepositoryMock({
      getAssetById: mockFn
    });
    const usecase = GetAssetTags({
      recordRepository: mockRecordRepository
    });
    // act
    const counts = await usecase([
      'monday1',
      'tuesday2',
      'wednesday3',
      'thursday4'
    ]);
    // assert
    expect(counts).toHaveLength(4);
    const mapping = new Map<string, number>();
    for (const item of counts) {
      mapping.set(item.label, item.count);
    }
    expect(mapping.get('bird')).toEqual(1);
    expect(mapping.get('cat')).toEqual(4);
    expect(mapping.get('dog')).toEqual(2);
    expect(mapping.get('mouse')).toEqual(1);
    expect(mockRecordRepository.getAssetById).toHaveBeenCalledTimes(4);
    mock.clearAllMocks();
  });
});
