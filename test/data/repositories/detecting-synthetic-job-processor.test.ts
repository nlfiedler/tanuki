//
// Copyright (c) 2026 Nathan Fiedler
//
import { describe, expect, mock, test } from 'bun:test';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { SyntheticJob } from 'tanuki/server/domain/entities/face.ts';
import {
  SyntheticData,
  SyntheticStatus
} from 'tanuki/server/domain/entities/synthetic-data.ts';
import { DetectingSyntheticJobProcessor } from 'tanuki/server/data/repositories/detecting-synthetic-job-processor.ts';
import { recordRepositoryMock } from '../../domain/usecases/mocking.ts';

function labelsJob(assetId: string): SyntheticJob {
  return new SyntheticJob(1, assetId, 'labels', 10, 0, null, 123);
}

function imageAsset(key: string): Asset {
  const a = new Asset(key);
  a.mediaType = 'image/jpeg';
  a.byteLength = 1024;
  return a;
}

describe('DetectingSyntheticJobProcessor', function () {
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
      syntheticDetector: { detectLabels }
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
      syntheticDetector: { detectLabels: mock(() => Promise.resolve([])) }
    });

    await sut.process(labelsJob('asset-2'));

    expect(captured.data).toBeNull();
    expect(captured.status).toEqual(SyntheticStatus.READY);
  });

  test('does nothing when the asset has been deleted', async function () {
    const recordRepository = recordRepositoryMock({
      getAssetById: mock(() => Promise.resolve(null))
    });
    const detectLabels = mock(() => Promise.resolve(['x']));
    const sut = new DetectingSyntheticJobProcessor({
      recordRepository,
      syntheticDetector: { detectLabels }
    });

    await sut.process(labelsJob('gone'));

    expect(detectLabels).toHaveBeenCalledTimes(0);
    expect(recordRepository.setSynthetic).toHaveBeenCalledTimes(0);
  });

  test('throws on an unsupported job kind so the pool can retry/fail it', async function () {
    const recordRepository = recordRepositoryMock({});
    const sut = new DetectingSyntheticJobProcessor({
      recordRepository,
      syntheticDetector: { detectLabels: mock(() => Promise.resolve([])) }
    });
    const facesJob = new SyntheticJob(2, 'a', 'faces', 0, 0, null, 1);
    expect(sut.process(facesJob)).rejects.toThrow('unsupported');
  });
});
