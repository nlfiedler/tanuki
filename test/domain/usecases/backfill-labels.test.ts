//
// Copyright (c) 2026 Nathan Fiedler
//
import { describe, expect, mock, test } from 'bun:test';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { SyntheticStatus } from 'tanuki/server/domain/entities/synthetic-data.ts';
import BackfillLabels from 'tanuki/server/domain/usecases/backfill-labels.ts';
import { faceStoreMock, recordRepositoryMock } from './mocking.ts';

function asset(key: string, mediaType: string): Asset {
  const a = new Asset(key);
  a.mediaType = mediaType;
  return a;
}

describe('BackfillLabels use case', function () {
  test('enqueues labels jobs only for unlabelled, unqueued images', async function () {
    // img1: pending, no job  -> enqueue
    // img2: already READY    -> skip
    // img3: pending, but a job is already queued -> skip
    // vid1: a video          -> skip (non-image)
    const page = [
      asset('img1', 'image/jpeg'),
      asset('img2', 'image/jpeg'),
      asset('img3', 'image/png'),
      asset('vid1', 'video/mp4')
    ];
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
          ['img1', SyntheticStatus.PENDING],
          ['img2', SyntheticStatus.READY],
          ['img3', SyntheticStatus.PENDING]
        ])
      )
    );
    const mockRecordRepository = recordRepositoryMock({
      fetchAssets,
      fetchSyntheticStatus
    });
    const hasPendingJob = mock((assetId: string) =>
      Promise.resolve(assetId === 'img3')
    );
    const mockFaceStore = faceStoreMock({ hasPendingJob });

    const usecase = BackfillLabels({
      recordRepository: mockRecordRepository,
      faceStore: mockFaceStore
    });
    const count = await usecase();

    expect(count).toEqual(1);
    expect(mockFaceStore.enqueueJob).toHaveBeenCalledTimes(1);
    expect(mockFaceStore.enqueueJob).toHaveBeenCalledWith('img1', 'labels');
    // status is only consulted for the four images' batch (one call)
    expect(mockRecordRepository.fetchSyntheticStatus).toHaveBeenCalledTimes(1);
  });

  test('returns zero when there are no assets', async function () {
    const mockRecordRepository = recordRepositoryMock({});
    const mockFaceStore = faceStoreMock({});
    const usecase = BackfillLabels({
      recordRepository: mockRecordRepository,
      faceStore: mockFaceStore
    });
    expect(await usecase()).toEqual(0);
    expect(mockFaceStore.enqueueJob).toHaveBeenCalledTimes(0);
  });
});
