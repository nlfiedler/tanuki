//
// Copyright (c) 2026 Nathan Fiedler
//
import { describe, expect, mock, test } from 'bun:test';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { SyntheticStatus } from 'tanuki/server/domain/entities/synthetic-data.ts';
import RetrySyntheticJobs from 'tanuki/server/domain/usecases/retry-synthetic-jobs.ts';
import { faceStoreMock, recordRepositoryMock } from './mocking.ts';

function asset(key: string): Asset {
  const a = new Asset(key);
  a.mediaType = 'image/jpeg';
  return a;
}

describe('RetrySyntheticJobs use case', function () {
  test('re-enqueues FAILED assets and resets them to PENDING', async function () {
    // a1: FAILED, no job   -> enqueue + reset
    // a2: READY            -> skip
    // a3: FAILED, has job  -> skip (already being retried)
    // a4: PENDING          -> skip
    const page = [asset('a1'), asset('a2'), asset('a3'), asset('a4')];
    let calls = 0;
    const fetchAssets = mock(() => {
      calls++;
      return Promise.resolve(
        calls === 1
          ? ([page, 'cursor'] as [Asset[], any])
          : ([[] as Asset[], 'done'] as [Asset[], any])
      );
    });
    const fetchSyntheticStatus = mock(() =>
      Promise.resolve(
        new Map([
          ['a1', SyntheticStatus.FAILED],
          ['a2', SyntheticStatus.READY],
          ['a3', SyntheticStatus.FAILED],
          ['a4', SyntheticStatus.PENDING]
        ])
      )
    );
    const mockRecordRepository = recordRepositoryMock({
      fetchAssets,
      fetchSyntheticStatus
    });
    const hasPendingJob = mock((assetId: string) =>
      Promise.resolve(assetId === 'a3')
    );
    const mockFaceStore = faceStoreMock({ hasPendingJob });

    const usecase = RetrySyntheticJobs({
      recordRepository: mockRecordRepository,
      faceStore: mockFaceStore
    });
    const count = await usecase();

    expect(count).toEqual(1);
    expect(mockFaceStore.enqueueJob).toHaveBeenCalledTimes(1);
    expect(mockFaceStore.enqueueJob).toHaveBeenCalledWith('a1', 'labels');
    expect(mockRecordRepository.setSynthetic).toHaveBeenCalledTimes(1);
    expect(mockRecordRepository.setSynthetic).toHaveBeenCalledWith(
      'a1',
      null,
      SyntheticStatus.PENDING
    );
  });

  test('kind=faces re-enqueues FAILED faces from the face store', async function () {
    // f1: FAILED, no job -> enqueue + reset; f2: FAILED but already queued -> skip
    const assetIdsWithFacesStatus = mock(() => Promise.resolve(['f1', 'f2']));
    const hasPendingJob = mock((assetId: string) =>
      Promise.resolve(assetId === 'f2')
    );
    const mockRecordRepository = recordRepositoryMock({});
    const mockFaceStore = faceStoreMock({
      assetIdsWithFacesStatus,
      hasPendingJob
    });
    const usecase = RetrySyntheticJobs({
      recordRepository: mockRecordRepository,
      faceStore: mockFaceStore
    });

    expect(await usecase('faces')).toEqual(1);
    // the labels scan is skipped entirely for kind=faces
    expect(mockRecordRepository.fetchAssets).toHaveBeenCalledTimes(0);
    expect(mockFaceStore.enqueueJob).toHaveBeenCalledWith('f1', 'faces');
    expect(mockFaceStore.setFacesStatus).toHaveBeenCalledWith(
      'f1',
      SyntheticStatus.PENDING
    );
  });

  test('an unknown kind is a no-op', async function () {
    const mockRecordRepository = recordRepositoryMock({});
    const mockFaceStore = faceStoreMock({});
    const usecase = RetrySyntheticJobs({
      recordRepository: mockRecordRepository,
      faceStore: mockFaceStore
    });
    expect(await usecase('bogus')).toEqual(0);
    expect(mockRecordRepository.fetchAssets).toHaveBeenCalledTimes(0);
    expect(mockFaceStore.assetIdsWithFacesStatus).toHaveBeenCalledTimes(0);
  });
});
