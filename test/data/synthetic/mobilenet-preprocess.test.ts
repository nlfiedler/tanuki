//
// Copyright (c) 2026 Nathan Fiedler
//
import { readFileSync } from 'node:fs';
import { describe, expect, test } from 'bun:test';
import {
  CROP,
  preprocessImage
} from 'tanuki/server/data/synthetic/mobilenet-preprocess.ts';

describe('MobileNetV2 preprocessing', function () {
  test('produces a normalized NCHW tensor of the expected size', async function () {
    const bytes = readFileSync('./test/fixtures/dcp_1069.jpg');
    const tensor = await preprocessImage(bytes);

    expect(tensor.length).toEqual(3 * CROP * CROP);
    // every value is finite and lands in a plausible post-normalization band
    let min = Number.POSITIVE_INFINITY;
    let max = Number.NEGATIVE_INFINITY;
    let allFinite = true;
    for (const v of tensor) {
      if (!Number.isFinite(v)) allFinite = false;
      if (v < min) min = v;
      if (v > max) max = v;
    }
    expect(allFinite).toBe(true);
    // ImageNet normalization pushes values roughly into [-2.2, 2.7]
    expect(min).toBeGreaterThan(-3);
    expect(max).toBeLessThan(3);
    // a real photograph spans both sides of zero
    expect(min).toBeLessThan(0);
    expect(max).toBeGreaterThan(0);
  });

  test('handles a tiny image by upscaling to the crop size', async function () {
    // a 10x10 solid image still yields a full 224x224 crop
    const sharpModule = await import('sharp');
    const tiny = await sharpModule.default({
      create: {
        width: 10,
        height: 10,
        channels: 3,
        background: { r: 120, g: 64, b: 200 }
      }
    })
      .jpeg()
      .toBuffer();
    const tensor = await preprocessImage(tiny);
    expect(tensor.length).toEqual(3 * CROP * CROP);
  });
});
