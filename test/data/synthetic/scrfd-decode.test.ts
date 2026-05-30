//
// Copyright (c) 2026 Nathan Fiedler
//
import { describe, expect, test } from 'bun:test';
import {
  decodeScrfd,
  nms,
  type RawDetection
} from 'tanuki/server/data/synthetic/scrfd-decode.ts';

/**
 * Build the three per-stride output arrays for a 16×16 detector input, with a
 * single positive anchor planted at a known cell. Strides are [8, 16, 32]; for
 * a 16×16 input the grids are 2×2, 1×1, 1×1, with 2 anchors each → 8, 2, 2
 * score entries.
 */
function buildOutputs(planted: {
  stride: 0 | 1 | 2;
  index: number;
  score: number;
  dist: [number, number, number, number];
}) {
  const counts = [8, 2, 2]; // anchors per stride for a 16px input
  const scores = counts.map((n) => new Float32Array(n));
  const bboxes = counts.map((n) => new Float32Array(n * 4));
  const kpss = counts.map((n) => new Float32Array(n * 10));
  scores[planted.stride]![planted.index] = planted.score;
  for (let k = 0; k < 4; k++) {
    bboxes[planted.stride]![planted.index * 4 + k] = planted.dist[k]!;
  }
  return { scores, bboxes, kpss };
}

describe('decodeScrfd', function () {
  test('thresholds and recovers a box from anchor + distances', function () {
    // stride 8, grid 2×2, anchor index 6 -> cell (i=1, j=1), anchor 0:
    // index = (i*width + j)*2 + a = (1*2 + 1)*2 + 0 = 6, center = (8, 8).
    const { scores, bboxes, kpss } = buildOutputs({
      stride: 0,
      index: 6,
      score: 0.9,
      // distances in stride units: left/top/right/bottom = 1,1,1,1 -> ±8 px
      dist: [1, 1, 1, 1]
    });
    const dets = decodeScrfd(scores, bboxes, kpss, 16, 16, 0.5);
    expect(dets).toHaveLength(1);
    expect(dets[0]!.score).toBeCloseTo(0.9);
    expect(dets[0]!.box).toEqual([0, 0, 16, 16]);
  });

  test('discards candidates below the score threshold', function () {
    const { scores, bboxes, kpss } = buildOutputs({
      stride: 0,
      index: 6,
      score: 0.49,
      dist: [1, 1, 1, 1]
    });
    expect(decodeScrfd(scores, bboxes, kpss, 16, 16, 0.5)).toHaveLength(0);
  });
});

function box(
  b: [number, number, number, number],
  score: number
): RawDetection {
  return { box: b, score, kps: [] };
}

describe('nms', function () {
  test('suppresses heavily overlapping lower-scoring boxes', function () {
    const a = box([0, 0, 10, 10], 0.9);
    const b = box([1, 1, 11, 11], 0.8); // ~0.68 IoU with a -> suppressed
    const c = box([100, 100, 110, 110], 0.7); // disjoint -> kept
    const kept = nms([a, b, c], 0.4);
    expect(kept.map((d) => d.score)).toEqual([0.9, 0.7]);
  });

  test('keeps boxes whose overlap is under the threshold', function () {
    const a = box([0, 0, 10, 10], 0.9);
    const b = box([8, 8, 18, 18], 0.8); // small overlap -> kept
    expect(nms([a, b], 0.4)).toHaveLength(2);
  });
});
