//
// Copyright (c) 2026 Nathan Fiedler
//
import { beforeEach, describe, expect, mock, test } from 'bun:test';
// prepare the test environment as early as possible
import 'tanuki/test/env.ts';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import {
  DetectedFace,
  SyntheticJob
} from 'tanuki/server/domain/entities/face.ts';
import {
  SyntheticData,
  SyntheticStatus
} from 'tanuki/server/domain/entities/synthetic-data.ts';
import { DetectingSyntheticJobProcessor } from 'tanuki/server/data/repositories/detecting-synthetic-job-processor.ts';
import { SqliteFaceStore } from 'tanuki/server/data/repositories/sqlite-face-store.ts';
import { EnvSettingsRepository } from 'tanuki/server/data/repositories/env-settings-repository.ts';
import {
  faceStoreMock,
  recordRepositoryMock
} from '../../domain/usecases/mocking.ts';

const settingsRepository: any = { getFloat: (_n: string, fb: number) => fb };

/** Build an L2-normalized embedding. */
function unit(values: number[]): Float32Array {
  const v = Float32Array.from(values);
  const norm = Math.hypot(...values) || 1;
  for (let i = 0; i < v.length; i++) v[i]! /= norm;
  return v;
}

function detected(embedding: Float32Array, score = 0.9): DetectedFace {
  return new DetectedFace(
    [0, 0, 10, 10],
    embedding,
    Uint8Array.from([1, 2, 3]),
    score,
    'mobilefacenet-v1'
  );
}

function detectorStub(over: any = {}) {
  return {
    detectLabels: mock(() => Promise.resolve([] as string[])),
    detectFaces: mock(() => Promise.resolve([] as DetectedFace[])),
    faceModelVersion: () => 'mobilefacenet-v1',
    ...over
  };
}

function imageAsset(key: string): Asset {
  const a = new Asset(key);
  a.mediaType = 'image/jpeg';
  a.byteLength = 1024;
  return a;
}

function labelsJob(assetId: string): SyntheticJob {
  return new SyntheticJob(1, assetId, 'labels', 10, 0, null, 123);
}

function facesJob(assetId: string): SyntheticJob {
  return new SyntheticJob(2, assetId, 'faces', 10, 0, null, 123);
}

describe('DetectingSyntheticJobProcessor', function () {
  describe('labels', function () {
    test('detects labels and persists them as READY', async function () {
      let captured: any = null;
      const recordRepository = recordRepositoryMock({
        getAssetById: mock(() => Promise.resolve(imageAsset('asset-1'))),
        setSynthetic: mock((id, data, status) => {
          captured = { id, data, status };
          return Promise.resolve();
        })
      });
      const detectLabels = mock(() => Promise.resolve(['beach', 'sky']));
      const sut = new DetectingSyntheticJobProcessor({
        recordRepository,
        faceStore: faceStoreMock({}),
        syntheticDetector: detectorStub({ detectLabels }),
        settingsRepository
      });

      await sut.process(labelsJob('asset-1'));

      expect(detectLabels).toHaveBeenCalledTimes(1);
      expect(captured.id).toEqual('asset-1');
      expect(captured.status).toEqual(SyntheticStatus.READY);
      expect(captured.data).toBeInstanceOf(SyntheticData);
      expect(captured.data.labels).toEqual(['beach', 'sky']);
      expect(captured.data.primaryLabel).toEqual('beach');
    });

    test('persists null data (still READY) when no labels clear the floor', async function () {
      let captured: any = null;
      const recordRepository = recordRepositoryMock({
        getAssetById: mock(() => Promise.resolve(imageAsset('asset-2'))),
        setSynthetic: mock((id, data, status) => {
          captured = { id, data, status };
          return Promise.resolve();
        })
      });
      const sut = new DetectingSyntheticJobProcessor({
        recordRepository,
        faceStore: faceStoreMock({}),
        syntheticDetector: detectorStub(),
        settingsRepository
      });

      await sut.process(labelsJob('asset-2'));

      expect(captured.data).toBeNull();
      expect(captured.status).toEqual(SyntheticStatus.READY);
    });
  });

  describe('faces', function () {
    const faceStore = new SqliteFaceStore({
      settingsRepository: new EnvSettingsRepository()
    });
    beforeEach(async function () {
      await faceStore.destroyAndCreate();
    });

    test('clusters detected faces and records READY', async function () {
      const detectFaces = mock(() =>
        Promise.resolve([
          detected(unit([1, 0, 0])),
          detected(unit([0.98, 0.02, 0])), // ~same person as the first
          detected(unit([0, 1, 0])) // a different person
        ])
      );
      const recordRepository = recordRepositoryMock({
        getAssetById: mock(() => Promise.resolve(imageAsset('asset-1')))
      });
      const sut = new DetectingSyntheticJobProcessor({
        recordRepository,
        faceStore,
        syntheticDetector: detectorStub({ detectFaces }),
        settingsRepository
      });

      await sut.process(facesJob('asset-1'));

      const people = await faceStore.listPeople(true);
      expect(people).toHaveLength(2);
      expect(people.map((p) => p.faceCount).sort()).toEqual([1, 2]);
      const status = await faceStore.fetchFacesStatus(['asset-1']);
      expect(status.get('asset-1')).toEqual(SyntheticStatus.READY);
    });

    test('reprocessing replaces prior faces rather than duplicating', async function () {
      const recordRepository = recordRepositoryMock({
        getAssetById: mock(() => Promise.resolve(imageAsset('asset-1')))
      });
      const detectFaces = mock(() =>
        Promise.resolve([detected(unit([1, 0, 0]))])
      );
      const sut = new DetectingSyntheticJobProcessor({
        recordRepository,
        faceStore,
        syntheticDetector: detectorStub({ detectFaces }),
        settingsRepository
      });

      await sut.process(facesJob('asset-1'));
      await sut.process(facesJob('asset-1'));

      const people = await faceStore.listPeople(true);
      expect(people).toHaveLength(1);
      const page = await faceStore.assetIdsByPerson(people[0]!.person.id, 0, 10);
      expect(page.total).toEqual(1);
    });

    test('records READY even when an image has no faces', async function () {
      const recordRepository = recordRepositoryMock({
        getAssetById: mock(() => Promise.resolve(imageAsset('asset-1')))
      });
      const sut = new DetectingSyntheticJobProcessor({
        recordRepository,
        faceStore,
        syntheticDetector: detectorStub(),
        settingsRepository
      });

      await sut.process(facesJob('asset-1'));

      expect(await faceStore.listPeople(true)).toHaveLength(0);
      const status = await faceStore.fetchFacesStatus(['asset-1']);
      expect(status.get('asset-1')).toEqual(SyntheticStatus.READY);
    });

    test('persists valid faces even if one fails (partial records)', async function () {
      const recordRepository = recordRepositoryMock({
        getAssetById: mock(() => Promise.resolve(imageAsset('asset-1')))
      });
      let inserts = 0;
      const setStatus = mock(() => Promise.resolve());
      const partialStore = faceStoreMock({
        nearestPerson: mock(() => Promise.resolve(null)),
        createPerson: mock(() => Promise.resolve({ id: `p${inserts}` } as any)),
        insertFace: mock(() => {
          inserts++;
          if (inserts === 2) return Promise.reject(new Error('bad embedding'));
          return Promise.resolve();
        }),
        setFacesStatus: setStatus
      });
      const detectFaces = mock(() =>
        Promise.resolve([
          detected(unit([1, 0, 0])),
          detected(unit([0, 1, 0])),
          detected(unit([0, 0, 1]))
        ])
      );
      const sut = new DetectingSyntheticJobProcessor({
        recordRepository,
        faceStore: partialStore,
        syntheticDetector: detectorStub({ detectFaces }),
        settingsRepository
      });

      await sut.process(facesJob('asset-1'));

      expect(partialStore.insertFace).toHaveBeenCalledTimes(3);
      // the failure is swallowed; the asset still completes READY
      expect(setStatus).toHaveBeenCalledWith('asset-1', SyntheticStatus.READY);
    });

    test('throws when every detected face fails to persist (no silent wipe)', async function () {
      const recordRepository = recordRepositoryMock({
        getAssetById: mock(() => Promise.resolve(imageAsset('asset-1')))
      });
      const setStatus = mock(() => Promise.resolve());
      const failingStore = faceStoreMock({
        nearestPerson: mock(() => Promise.resolve(null)),
        createPerson: mock(() => Promise.resolve({ id: 'p' } as any)),
        insertFace: mock(() => Promise.reject(new Error('store offline'))),
        setFacesStatus: setStatus
      });
      const detectFaces = mock(() =>
        Promise.resolve([detected(unit([1, 0, 0])), detected(unit([0, 1, 0]))])
      );
      const sut = new DetectingSyntheticJobProcessor({
        recordRepository,
        faceStore: failingStore,
        syntheticDetector: detectorStub({ detectFaces }),
        settingsRepository
      });

      // the whole batch failed -> throw so the pool retries, rather than
      // recording READY with zero faces (which would also wipe prior faces)
      await expect(sut.process(facesJob('asset-1'))).rejects.toThrow(
        'failed to persist'
      );
      expect(setStatus).not.toHaveBeenCalled();
    });
  });

  describe('common', function () {
    test('does nothing when the asset has been deleted', async function () {
      const recordRepository = recordRepositoryMock({
        getAssetById: mock(() => Promise.resolve(null))
      });
      const detectLabels = mock(() => Promise.resolve(['x']));
      const sut = new DetectingSyntheticJobProcessor({
        recordRepository,
        faceStore: faceStoreMock({}),
        syntheticDetector: detectorStub({ detectLabels }),
        settingsRepository
      });

      await sut.process(labelsJob('gone'));

      expect(detectLabels).toHaveBeenCalledTimes(0);
      expect(recordRepository.setSynthetic).toHaveBeenCalledTimes(0);
    });

    test('throws on an unsupported job kind so the pool can retry/fail it', async function () {
      const recordRepository = recordRepositoryMock({
        getAssetById: mock(() => Promise.resolve(imageAsset('a')))
      });
      const sut = new DetectingSyntheticJobProcessor({
        recordRepository,
        faceStore: faceStoreMock({}),
        syntheticDetector: detectorStub(),
        settingsRepository
      });
      const badJob = new SyntheticJob(2, 'a', 'other' as any, 0, 0, null, 1);
      expect(sut.process(badJob)).rejects.toThrow('unsupported');
    });
  });
});
