//
// Copyright (c) 2026 Nathan Fiedler
//
import { describe, expect, mock, test } from 'bun:test';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { AssetMetadata } from 'tanuki/server/domain/entities/asset-metadata.ts';
import BackfillImageMetadata from 'tanuki/server/domain/usecases/backfill-image-metadata.ts';
import BackfillVideoMetadata from 'tanuki/server/domain/usecases/backfill-video-metadata.ts';
import { blobRepositoryMock, recordRepositoryMock } from './mocking.ts';

function makeAsset(key: string, mediaType: string): Asset {
  const asset = new Asset(key);
  asset.checksum = 'sha256-test';
  asset.filename = 'fixture';
  asset.byteLength = 1;
  asset.mediaType = mediaType;
  return asset;
}

function pageOnce(assets: Asset[]) {
  const fn = mock();
  fn.mockImplementationOnce(() =>
    Promise.resolve([assets, 'done'] as [Asset[], any])
  );
  fn.mockImplementation(() =>
    Promise.resolve([[], 'done'] as [Asset[], any])
  );
  return fn;
}

describe('BackfillImageMetadata', function () {
  test('populates metadata for image assets without it', async function () {
    const stored: Asset[] = [];
    const image = makeAsset('img1', 'image/jpeg');
    const recordRepository = recordRepositoryMock({
      fetchAssets: pageOnce([image]),
      putAsset: mock((asset: Asset) => {
        stored.push(asset);
        return Promise.resolve();
      })
    });
    const blobRepository = blobRepositoryMock({
      fetchMetadata: mock(() =>
        Promise.resolve({
          Make: { description: 'Canon' },
          Model: { description: 'EOS 5D' },
          FNumber: { description: '2.8' },
          ISOSpeedRatings: { value: 200 }
        })
      )
    });
    const usecase = BackfillImageMetadata({
      recordRepository,
      blobRepository
    });
    const count = await usecase();
    expect(count).toEqual(1);
    expect(stored).toHaveLength(1);
    expect(stored[0]!.metadata?.cameraMake).toEqual('Canon');
    expect(stored[0]!.metadata?.cameraModel).toEqual('EOS 5D');
    expect(stored[0]!.metadata?.fNumber).toBeCloseTo(2.8);
    expect(stored[0]!.metadata?.iso).toEqual(200);
    mock.clearAllMocks();
  });

  test('skips non-image assets and already-populated metadata', async function () {
    const video = makeAsset('vid1', 'video/mp4');
    const already = makeAsset('img2', 'image/jpeg');
    already.metadata = new AssetMetadata();
    already.metadata.cameraMake = 'Nikon';
    const recordRepository = recordRepositoryMock({
      fetchAssets: pageOnce([video, already]),
      putAsset: mock(() => Promise.resolve())
    });
    const blobRepository = blobRepositoryMock({
      fetchMetadata: mock(() => Promise.resolve({ Make: { description: 'X' } }))
    });
    const usecase = BackfillImageMetadata({
      recordRepository,
      blobRepository
    });
    const count = await usecase();
    expect(count).toEqual(0);
    expect(blobRepository.fetchMetadata).toHaveBeenCalledTimes(0);
    mock.clearAllMocks();
  });
});

describe('BackfillVideoMetadata', function () {
  test('populates metadata for video assets without it', async function () {
    const stored: Asset[] = [];
    const video = makeAsset('vid1', 'video/mp4');
    const recordRepository = recordRepositoryMock({
      fetchAssets: pageOnce([video]),
      putAsset: mock((asset: Asset) => {
        stored.push(asset);
        return Promise.resolve();
      })
    });
    const blobRepository = blobRepositoryMock({
      fetchMetadata: mock(() =>
        Promise.resolve({
          format: { duration: '30.0' },
          streams: [
            {
              codec_type: 'video',
              codec_name: 'h264',
              width: 1920,
              height: 1080,
              r_frame_rate: '30/1'
            }
          ]
        })
      )
    });
    const usecase = BackfillVideoMetadata({
      recordRepository,
      blobRepository
    });
    const count = await usecase();
    expect(count).toEqual(1);
    expect(stored).toHaveLength(1);
    expect(stored[0]!.metadata?.videoCodec).toEqual('h264');
    expect(stored[0]!.metadata?.duration).toBeCloseTo(30);
    expect(stored[0]!.metadata?.displayWidth).toEqual(1920);
    expect(stored[0]!.metadata?.frameRate).toBeCloseTo(30);
    mock.clearAllMocks();
  });
});
