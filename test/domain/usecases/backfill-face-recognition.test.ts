//
// Copyright (c) 2026 Nathan Fiedler
//
import { describe, expect, mock, test } from 'bun:test';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { SyntheticStatus } from 'tanuki/server/domain/entities/synthetic-data.ts';
import BackfillFaceRecognition from 'tanuki/server/domain/usecases/backfill-face-recognition.ts';
import { faceStoreMock, recordRepositoryMock } from './mocking.ts';

function image(key: string): Asset {
  const a = new Asset(key);
  a.mediaType = 'image/jpeg';
  return a;
}

function video(key: string): Asset {
  const a = new Asset(key);
  a.mediaType = 'video/mp4';
  return a;
}

const detector: any = { faceModelVersion: () => 'mobilefacenet-v1' };

describe('BackfillFaceRecognition use case', function () {
  test('enqueues only images needing current face data, idempotently', async function () {
    // i1: never processed (PENDING)     -> enqueue
    // i2: READY, current version        -> skip
    // i3: READY but stale model version -> enqueue (reprocess)
    // i4: PENDING but already queued     -> skip
    // v1: a video                        -> ignored
    const page = [image('i1'), image('i2'), image('i3'), image('i4'), video('v1')];
    let calls = 0;
    const fetchAssets = mock(() => {
      calls++;
      return Promise.resolve(
        calls === 1
          ? ([page, 'cursor'] as [Asset[], any])
          : ([[] as Asset[], 'done'] as [Asset[], any])
      );
    });
    const fetchFacesStatus = mock(() =>
      Promise.resolve(
        new Map([
          ['i1', SyntheticStatus.PENDING],
          ['i2', SyntheticStatus.READY],
          ['i3', SyntheticStatus.READY],
          ['i4', SyntheticStatus.PENDING]
        ])
      )
    );
    const modelVersionsByAssets = mock(() =>
      Promise.resolve(
        new Map([
          ['i2', new Set(['mobilefacenet-v1'])],
          ['i3', new Set(['mobilefacenet-v0'])] // stale
        ])
      )
    );
    const hasPendingJob = mock((assetId: string) =>
      Promise.resolve(assetId === 'i4')
    );
    const mockFaceStore = faceStoreMock({
      fetchFacesStatus,
      modelVersionsByAssets,
      hasPendingJob
    });
    const mockRecordRepository = recordRepositoryMock({ fetchAssets });

    const usecase = BackfillFaceRecognition({
      recordRepository: mockRecordRepository,
      faceStore: mockFaceStore,
      syntheticDetector: detector
    });
    const count = await usecase();

    expect(count).toEqual(2);
    expect(mockFaceStore.enqueueJob).toHaveBeenCalledWith('i1', 'faces');
    expect(mockFaceStore.enqueueJob).toHaveBeenCalledWith('i3', 'faces');
    expect(mockFaceStore.enqueueJob).toHaveBeenCalledTimes(2);
  });

  test('force re-enqueues every image regardless of status', async function () {
    const page = [image('i1'), image('i2'), image('i3'), video('v1')];
    let calls = 0;
    const fetchAssets = mock(() => {
      calls++;
      return Promise.resolve(
        calls === 1
          ? ([page, 'cursor'] as [Asset[], any])
          : ([[] as Asset[], 'done'] as [Asset[], any])
      );
    });
    const fetchFacesStatus = mock(() => Promise.resolve(new Map()));
    const modelVersionsByAssets = mock(() => Promise.resolve(new Map()));
    // i2 already has a queued job and is still skipped even under force
    const hasPendingJob = mock((assetId: string) =>
      Promise.resolve(assetId === 'i2')
    );
    const mockFaceStore = faceStoreMock({
      fetchFacesStatus,
      modelVersionsByAssets,
      hasPendingJob
    });
    const mockRecordRepository = recordRepositoryMock({ fetchAssets });

    const usecase = BackfillFaceRecognition({
      recordRepository: mockRecordRepository,
      faceStore: mockFaceStore,
      syntheticDetector: detector
    });
    const count = await usecase(true);

    expect(count).toEqual(2); // i1 + i3 (i2 skipped: already queued; v1 is a video)
    // force skips the status/version reads entirely
    expect(fetchFacesStatus).toHaveBeenCalledTimes(0);
    expect(modelVersionsByAssets).toHaveBeenCalledTimes(0);
  });
});
