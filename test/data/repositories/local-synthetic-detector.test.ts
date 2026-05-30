//
// Copyright (c) 2026 Nathan Fiedler
//
import { existsSync, readFileSync } from 'node:fs';
import { describe, expect, mock, test } from 'bun:test';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { LocalSyntheticDetector } from 'tanuki/server/data/repositories/local-synthetic-detector.ts';
import { MAX_LABELS } from 'tanuki/server/data/synthetic/label-curation.ts';

const MODEL = 'models/mobilenet_v2.onnx';
const hasModel = existsSync(MODEL);

// minimal settings stub: no overrides, so the default model path is used
const settingsRepository: any = { get() {} };

function detectorFor(fixturePath: string): {
  detector: LocalSyntheticDetector;
  asset: Asset;
} {
  const bytes = readFileSync(fixturePath);
  const asset = new Asset('asset-1');
  asset.mediaType = 'image/jpeg';
  asset.byteLength = bytes.length;
  const blobRepository: any = {
    fetchRange: mock(() => Promise.resolve(bytes))
  };
  const detector = new LocalSyntheticDetector({
    blobRepository,
    settingsRepository
  });
  return { detector, asset };
}

describe('LocalSyntheticDetector', function () {
  test('returns an empty list for non-image assets without loading the model', async function () {
    const blobRepository: any = { fetchRange: mock(() => Promise.resolve(Buffer.alloc(0))) };
    const detector = new LocalSyntheticDetector({
      blobRepository,
      settingsRepository
    });
    const video = new Asset('vid');
    video.mediaType = 'video/mp4';
    video.byteLength = 999;

    expect(await detector.detectLabels(video)).toEqual([]);
    expect(blobRepository.fetchRange).toHaveBeenCalledTimes(0);
  });

  // end-to-end against the real ONNX model; skipped when models aren't fetched
  (hasModel ? test : test.skip)(
    'classifies a real photo into curated labels',
    async function () {
      const { detector, asset } = detectorFor('./test/fixtures/dcp_1069.jpg');
      const labels = await detector.detectLabels(asset);

      expect(Array.isArray(labels)).toBe(true);
      expect(labels.length).toBeLessThanOrEqual(MAX_LABELS);
      for (const label of labels) expect(typeof label).toBe('string');
      // a clear photograph should produce at least one label above the floor
      expect(labels.length).toBeGreaterThan(0);
      // surfaced for manual inspection of detector quality
      console.log('dcp_1069.jpg labels:', labels);
    },
    30_000
  );
});
