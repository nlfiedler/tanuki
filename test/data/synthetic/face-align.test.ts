//
// Copyright (c) 2026 Nathan Fiedler
//
import { describe, expect, test } from 'bun:test';
import {
  type Affine,
  arcfacePreprocess,
  estimateSimilarityTransform,
  invertAffine,
  l2normalize,
  warpAffineBilinear
} from 'tanuki/server/data/synthetic/face-align.ts';

/** Apply a 2×3 affine to a point. */
function apply(m: Affine, x: number, y: number): [number, number] {
  return [m[0][0] * x + m[0][1] * y + m[0][2], m[1][0] * x + m[1][1] * y + m[1][2]];
}

describe('estimateSimilarityTransform', function () {
  test('recovers a known scale + rotation + translation exactly', function () {
    // Ground-truth similarity: scale 2, rotate 90°, translate (5, -3).
    // [[0,-2,5],[2,0,-3]]
    const src = [0, 0, 1, 0, 0, 1, 1, 1, 2, 2];
    const truth: Affine = [
      [0, -2, 5],
      [2, 0, -3]
    ];
    const dst: number[] = [];
    for (let i = 0; i < src.length; i += 2) {
      const [X, Y] = apply(truth, src[i]!, src[i + 1]!);
      dst.push(X, Y);
    }
    const m = estimateSimilarityTransform(src, dst);
    expect(m[0][0]).toBeCloseTo(0, 5);
    expect(m[0][1]).toBeCloseTo(-2, 5);
    expect(m[0][2]).toBeCloseTo(5, 5);
    expect(m[1][0]).toBeCloseTo(2, 5);
    expect(m[1][1]).toBeCloseTo(0, 5);
    expect(m[1][2]).toBeCloseTo(-3, 5);
  });

  test('the transform maps source landmarks onto the destination', function () {
    const src = [10, 12, 40, 14, 25, 30, 13, 48, 38, 49];
    const dst = [38.29, 51.69, 73.53, 51.5, 56.02, 71.73, 41.54, 92.36, 70.72, 92.2];
    const m = estimateSimilarityTransform(src, dst);
    // similarity can't fit 5 arbitrary points perfectly, but the mean should land
    let mx = 0;
    let my = 0;
    for (let i = 0; i < src.length; i += 2) {
      const [X, Y] = apply(m, src[i]!, src[i + 1]!);
      mx += X;
      my += Y;
    }
    let dmx = 0;
    let dmy = 0;
    for (let i = 0; i < dst.length; i += 2) {
      dmx += dst[i]!;
      dmy += dst[i + 1]!;
    }
    expect(mx / 5).toBeCloseTo(dmx / 5, 3);
    expect(my / 5).toBeCloseTo(dmy / 5, 3);
  });
});

describe('invertAffine', function () {
  test('composes with the forward transform to the identity', function () {
    const m: Affine = [
      [0.5, -0.3, 10],
      [0.3, 0.5, -4]
    ];
    const inv = invertAffine(m);
    const [x, y] = apply(m, 7, 11);
    const [bx, by] = apply(inv, x, y);
    expect(bx).toBeCloseTo(7, 6);
    expect(by).toBeCloseTo(11, 6);
  });

  test('throws on a degenerate transform', function () {
    expect(() =>
      invertAffine([
        [0, 0, 1],
        [0, 0, 1]
      ])
    ).toThrow();
  });
});

describe('warpAffineBilinear', function () {
  test('an identity transform copies pixels through', function () {
    // 2×2 RGB image; identity src->dst means out == in for a 2×2 crop.
    const src = Uint8Array.from([
      10, 10, 10, 20, 20, 20, 30, 30, 30, 40, 40, 40
    ]);
    const identity: Affine = [
      [1, 0, 0],
      [0, 1, 0]
    ];
    const out = warpAffineBilinear(src, 2, 2, identity, 2, 2);
    expect([...out]).toEqual([...src]);
  });

  test('bilinearly samples a translated half-pixel', function () {
    // horizontal gradient 0..100 over 2 px; shifting by +0.5 px reads the mean.
    const src = Uint8Array.from([0, 0, 0, 100, 100, 100]);
    // dst->src adds 0.5 to x, so forward src->dst subtracts 0.5: [[1,0,-0.5],...]
    const m: Affine = [
      [1, 0, -0.5],
      [0, 1, 0]
    ];
    const out = warpAffineBilinear(src, 2, 1, m, 1, 1);
    expect(out[0]).toBe(50);
  });
});

describe('arcfacePreprocess', function () {
  test('normalizes to (v-127.5)/128 in planar NCHW order', function () {
    const rgb = new Uint8Array(112 * 112 * 3).fill(128);
    const t = arcfacePreprocess(rgb);
    expect(t).toHaveLength(3 * 112 * 112);
    // (128 - 127.5) / 128 = 0.00390625 for every channel
    expect(t[0]).toBeCloseTo(0.003_906_25, 6);
    expect(t[112 * 112]).toBeCloseTo(0.003_906_25, 6);
  });
});

describe('l2normalize', function () {
  test('produces a unit vector', function () {
    const v = l2normalize(Float32Array.from([3, 4]));
    expect(v[0]).toBeCloseTo(0.6, 6);
    expect(v[1]).toBeCloseTo(0.8, 6);
  });

  test('leaves a zero vector unchanged', function () {
    const v = l2normalize(Float32Array.from([0, 0, 0]));
    expect([...v]).toEqual([0, 0, 0]);
  });
});
