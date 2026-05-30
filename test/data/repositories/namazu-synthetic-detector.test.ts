//
// Copyright (c) 2026 Nathan Fiedler
//
import { afterEach, describe, expect, mock, test } from 'bun:test';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { NamazuSyntheticDetector } from 'tanuki/server/data/repositories/namazu-synthetic-detector.ts';

const settingsRepository: any = {
  get: (name: string) => (name === 'NAMAZU_URL' ? 'http://namazu.test/' : undefined)
};

function detector(): NamazuSyntheticDetector {
  return new NamazuSyntheticDetector({ settingsRepository });
}

function imageAsset(key = 'asset-1'): Asset {
  const a = new Asset(key);
  a.mediaType = 'image/jpeg';
  a.byteLength = 1024;
  return a;
}

/** base64 of a little-endian Float32 vector. */
function embedB64(values: number[]): string {
  const f = Float32Array.from(values);
  return Buffer.from(f.buffer, f.byteOffset, f.byteLength).toString('base64');
}

/** Install a fetch stub returning the given response shape; returns the mock. */
function stubFetch(impl: (url: string, init?: any) => any) {
  const fn = mock(impl);
  // @ts-expect-error overriding the global for the test
  globalThis.fetch = fn;
  return fn;
}

const realFetch = globalThis.fetch;
afterEach(() => {
  globalThis.fetch = realFetch;
});

describe('NamazuSyntheticDetector', function () {
  test('non-image assets short-circuit without calling Namazu', async function () {
    const fn = stubFetch(() => {
      throw new Error('should not be called');
    });
    const video = new Asset('v');
    video.mediaType = 'video/mp4';
    video.byteLength = 99;
    expect(await detector().detectLabels(video)).toEqual([]);
    expect(await detector().detectFaces(video)).toEqual([]);
    expect(fn).toHaveBeenCalledTimes(0);
  });

  test('detectLabels returns curated names, deduped and sorted by score', async function () {
    stubFetch(() => ({
      status: 200,
      ok: true,
      json: async () => ({
        labels: [
          { name: 'beach', score: 0.4 },
          { name: 'sunset', score: 0.9 },
          { name: 'beach', score: 0.7 } // dup, higher score wins
        ]
      })
    }));
    expect(await detector().detectLabels(imageAsset())).toEqual([
      'sunset',
      'beach'
    ]);
  });

  test('detectFaces decodes embeddings and thumbnails', async function () {
    const thumb = Uint8Array.from([0xFF, 0xD8, 0xAB]);
    stubFetch((url: string) => {
      expect(url).toEqual('http://namazu.test/synthetic/asset-1');
      return {
        status: 200,
        ok: true,
        json: async () => ({
          faces: [
            {
              bbox: [1, 2, 3, 4],
              embedding: embedB64([3, 4, 0]),
              thumbnail: Buffer.from(thumb).toString('base64'),
              score: 0.97,
              model_version: 'mobilefacenet-v1'
            }
          ]
        })
      };
    });
    const faces = await detector().detectFaces(imageAsset());
    expect(faces).toHaveLength(1);
    expect(faces[0]!.bbox).toEqual([1, 2, 3, 4]);
    expect(faces[0]!.score).toBeCloseTo(0.97);
    expect(faces[0]!.modelVersion).toEqual('mobilefacenet-v1');
    // embedding is L2-normalized: [3,4,0] -> [0.6,0.8,0]
    expect(faces[0]!.embedding[0]).toBeCloseTo(0.6, 5);
    expect(faces[0]!.embedding[1]).toBeCloseTo(0.8, 5);
    expect([...faces[0]!.thumbnail]).toEqual([0xFF, 0xD8, 0xAB]);
  });

  test('a 204 response means no synthetic data', async function () {
    stubFetch(() => ({ status: 204, ok: false }));
    expect(await detector().detectLabels(imageAsset())).toEqual([]);
    expect(await detector().detectFaces(imageAsset())).toEqual([]);
  });

  test('a 4xx response throws so the worker can retry/fail', async function () {
    stubFetch(() => ({
      status: 422,
      ok: false,
      text: async () => 'extractor failure'
    }));
    expect(detector().detectFaces(imageAsset())).rejects.toThrow('422');
  });

  test('faceModelVersion defaults to the shared identifier', function () {
    expect(detector().faceModelVersion()).toEqual('mobilefacenet-v1');
  });

  test('coalesces labels + faces for one asset into a single request', async function () {
    const fn = stubFetch(() => ({
      status: 200,
      ok: true,
      json: async () => ({
        labels: [{ name: 'coat', score: 0.5 }],
        faces: []
      })
    }));
    // both jobs run against the same detector instance close together
    const det = detector();
    const asset = imageAsset('asset-1');
    await det.detectLabels(asset);
    await det.detectFaces(asset);
    // one /synthetic round-trip serves both, not two
    expect(fn).toHaveBeenCalledTimes(1);
  });

  test('does not cache a failed request', async function () {
    let calls = 0;
    stubFetch(() => {
      calls++;
      return calls === 1
        ? { status: 500, ok: false, text: async () => 'boom' }
        : { status: 200, ok: true, json: async () => ({ labels: [], faces: [] }) };
    });
    const det = detector();
    const asset = imageAsset('asset-1');
    await expect(det.detectFaces(asset)).rejects.toThrow('500');
    // a retry re-hits Namazu rather than replaying the cached rejection
    expect(await det.detectFaces(asset)).toEqual([]);
    expect(calls).toEqual(2);
  });
});
