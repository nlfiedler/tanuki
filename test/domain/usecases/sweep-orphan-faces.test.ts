//
// Copyright (c) 2026 Nathan Fiedler
//
import { describe, expect, mock, test } from 'bun:test';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import SweepOrphanFaces from 'tanuki/server/domain/usecases/sweep-orphan-faces.ts';
import { faceStoreMock, recordRepositoryMock } from './mocking.ts';

describe('SweepOrphanFaces use case', function () {
  test('removes faces only for assets that no longer exist', async function () {
    const allFaceAssetIds = mock(() =>
      Promise.resolve(['present-1', 'gone-1', 'present-2', 'gone-2'])
    );
    const getAssetById = mock((id: string) =>
      Promise.resolve(id.startsWith('present') ? new Asset(id) : null)
    );
    const mockFaceStore = faceStoreMock({ allFaceAssetIds });
    const mockRecordRepository = recordRepositoryMock({ getAssetById });

    const usecase = SweepOrphanFaces({
      recordRepository: mockRecordRepository,
      faceStore: mockFaceStore
    });
    const removed = await usecase();

    expect(removed).toEqual(2);
    expect(mockFaceStore.deleteByAssetId).toHaveBeenCalledTimes(2);
    expect(mockFaceStore.deleteByAssetId).toHaveBeenCalledWith('gone-1');
    expect(mockFaceStore.deleteByAssetId).toHaveBeenCalledWith('gone-2');
  });

  test('does nothing when every asset still exists', async function () {
    const allFaceAssetIds = mock(() => Promise.resolve(['a', 'b']));
    const getAssetById = mock((id: string) => Promise.resolve(new Asset(id)));
    const mockFaceStore = faceStoreMock({ allFaceAssetIds });
    const usecase = SweepOrphanFaces({
      recordRepository: recordRepositoryMock({ getAssetById }),
      faceStore: mockFaceStore
    });

    expect(await usecase()).toEqual(0);
    expect(mockFaceStore.deleteByAssetId).toHaveBeenCalledTimes(0);
  });
});
