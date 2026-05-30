//
// Copyright (c) 2026 Nathan Fiedler
//
import { readFileSync } from 'node:fs';
import path from 'node:path';

/** One entry of the ImageNet-class → display-label curation map. */
export interface LabelEntry {
  /** Original ImageNet class name (informational). */
  raw: string;
  /** Curated display label, or `null` to drop the class entirely. */
  label: string | null;
  /** Internal grouping (e.g. `animal`, `person`); not exposed via GraphQL. */
  category: string;
}

/** Scores at or below this softmax probability are floored out as noise. */
export const SCORE_FLOOR = 0.05;
/** Maximum number of display labels emitted per asset. */
export const MAX_LABELS = 20;

let cachedMap: Map<number, LabelEntry> | null = null;

/**
 * Load the curated label map (`labels-map.json`, keyed by ImageNet class
 * index). Cached after first read. The same file is shipped to Namazu so both
 * backends produce identical display labels.
 */
export function loadLabelMap(): Map<number, LabelEntry> {
  if (cachedMap !== null) return cachedMap;
  const file = path.join(import.meta.dir, 'labels-map.json');
  const raw = JSON.parse(readFileSync(file, 'utf8')) as Record<
    string,
    LabelEntry
  >;
  const map = new Map<number, LabelEntry>();
  for (const [index, entry] of Object.entries(raw)) {
    map.set(Number.parseInt(index, 10), entry);
  }
  cachedMap = map;
  return map;
}

/**
 * Numerically stable softmax over a vector of logits.
 *
 * @param logits - raw model outputs.
 * @returns probabilities summing to 1.
 */
export function softmax(logits: Float32Array | number[]): Float32Array {
  let max = Number.NEGATIVE_INFINITY;
  for (const value of logits) {
    if (value > max) max = value;
  }
  const exps = new Float32Array(logits.length);
  let sum = 0;
  for (const [i, value] of logits.entries()) {
    const e = Math.exp(value - max);
    exps[i] = e;
    sum += e;
  }
  for (const [i, e] of exps.entries()) exps[i] = e / sum;
  return exps;
}

/**
 * Apply the curation pipeline to per-class softmax probabilities:
 *
 * 1. drop scores below {@link SCORE_FLOOR};
 * 2. map each surviving class index through the label map;
 * 3. drop classes mapped to `null`;
 * 4. drop classes in the `person` category (handled by face recognition);
 * 5. de-duplicate by display label, keeping the maximum score per label;
 * 6. sort by score descending and cap at {@link MAX_LABELS}.
 *
 * @param probs - per-class softmax probabilities (length 1000).
 * @param map - curation map; defaults to the bundled one.
 * @returns ordered display labels (highest-confidence first).
 */
export function curateScores(
  probs: Float32Array | number[],
  map: Map<number, LabelEntry> = loadLabelMap()
): string[] {
  const byLabel = new Map<string, number>();
  for (const [i, score] of probs.entries()) {
    if (score < SCORE_FLOOR) continue;
    const entry = map.get(i);
    if (!entry || entry.label === null || entry.category === 'person') continue;
    const prev = byLabel.get(entry.label);
    if (prev === undefined || score > prev) {
      byLabel.set(entry.label, score);
    }
  }
  return [...byLabel.entries()]
    .sort((a, b) => b[1] - a[1])
    .slice(0, MAX_LABELS)
    .map(([label]) => label);
}

/**
 * Convenience wrapper: softmax the logits, then curate.
 *
 * @param logits - raw model outputs (length 1000).
 * @param map - curation map; defaults to the bundled one.
 * @returns ordered display labels.
 */
export function curateLogits(
  logits: Float32Array | number[],
  map?: Map<number, LabelEntry>
): string[] {
  return curateScores(softmax(logits), map);
}
