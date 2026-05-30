//
// Copyright (c) 2026 Nathan Fiedler
//
import { describe, expect, test } from 'bun:test';
import {
  curateScores,
  loadLabelMap,
  MAX_LABELS,
  SCORE_FLOOR,
  softmax,
  type LabelEntry
} from 'tanuki/server/data/synthetic/label-curation.ts';

const map = loadLabelMap();

/** Build a length-1000 probability vector from index→score pairs. */
function probs(pairs: Record<number, number>): Float32Array {
  const a = new Float32Array(1000);
  for (const [index, score] of Object.entries(pairs)) a[Number(index)] = score;
  return a;
}

function firstIndex(pred: (entry: LabelEntry) => boolean): number {
  for (const [index, entry] of map) if (pred(entry)) return index;
  throw new Error('no matching label-map entry');
}

describe('label curation', function () {
  test('the bundled map covers all 1000 ImageNet classes', function () {
    expect(map.size).toEqual(1000);
  });

  test('keeps a class above the floor and drops one below it', function () {
    const idx = firstIndex((e) => e.label !== null && e.category !== 'person');
    const label = map.get(idx)!.label!;
    expect(curateScores(probs({ [idx]: 0.9 }))).toEqual([label]);
    expect(curateScores(probs({ [idx]: SCORE_FLOOR - 0.001 }))).toEqual([]);
  });

  test('drops null-mapped classes', function () {
    // the bundled map happens to map every class, so exercise the null-drop
    // path with a purpose-built map
    const customMap = new Map<number, LabelEntry>([
      [0, { raw: 'noise', label: null, category: 'object' }],
      [1, { raw: 'keeper', label: 'kept', category: 'object' }]
    ]);
    expect(curateScores(probs({ 0: 0.9, 1: 0.9 }), customMap)).toEqual([
      'kept'
    ]);
  });

  test('drops person-category classes (handled by face recognition)', function () {
    const personIdx = firstIndex((e) => e.category === 'person');
    expect(curateScores(probs({ [personIdx]: 0.99 }))).toEqual([]);
  });

  test('de-duplicates by display label, keeping the max score', function () {
    // find a label produced by at least two distinct class indices
    const byLabel = new Map<string, number[]>();
    for (const [index, entry] of map) {
      if (entry.label === null || entry.category === 'person') continue;
      const list = byLabel.get(entry.label) ?? [];
      list.push(index);
      byLabel.set(entry.label, list);
    }
    const dup = [...byLabel.entries()].find(([, idxs]) => idxs.length >= 2)!;
    const [label, [a, b]] = dup;
    const result = curateScores(probs({ [a!]: 0.3, [b!]: 0.7 }));
    expect(result).toEqual([label]);
  });

  test('sorts by score descending', function () {
    const idxs = [...byDistinctLabel()].slice(0, 3);
    const [lo, mid, hi] = idxs;
    const result = curateScores(
      probs({ [lo!.index]: 0.2, [mid!.index]: 0.5, [hi!.index]: 0.8 })
    );
    expect(result).toEqual([hi!.label, mid!.label, lo!.label]);
  });

  test('caps the result at MAX_LABELS', function () {
    const pairs: Record<number, number> = {};
    let score = 0.99;
    for (const { index } of [...byDistinctLabel()].slice(0, MAX_LABELS + 5)) {
      pairs[index] = score;
      score -= 0.001;
    }
    expect(curateScores(probs(pairs)).length).toEqual(MAX_LABELS);
  });
});

describe('softmax', function () {
  test('produces a probability distribution', function () {
    const out = softmax([1, 2, 3]);
    const sum = out.reduce((acc, v) => acc + v, 0);
    expect(sum).toBeCloseTo(1, 5);
    expect(out[2]!).toBeGreaterThan(out[0]!);
  });
});

/** Distinct (index, label) pairs, one index per display label. */
function* byDistinctLabel(): Generator<{ index: number; label: string }> {
  const seen = new Set<string>();
  for (const [index, entry] of map) {
    if (entry.label === null || entry.category === 'person') continue;
    if (seen.has(entry.label)) continue;
    seen.add(entry.label);
    yield { index, label: entry.label };
  }
}
