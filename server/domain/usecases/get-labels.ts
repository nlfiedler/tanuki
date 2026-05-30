//
// Copyright (c) 2026 Nathan Fiedler
//
import assert from 'node:assert';
import { LabelEntry } from 'tanuki/server/domain/entities/synthetic-data.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';

export default ({
  recordRepository
}: {
  recordRepository: RecordRepository;
}) => {
  assert.ok(recordRepository, 'record repository must be defined');
  /**
   * Build the Labels-page index: one entry per distinct `primaryLabel`, with
   * its asset count and the id of the most-recent asset carrying that label
   * (the thumbnail). The per-label representative lookup uses the bounded
   * `latestAssetByLabel` (single row per round-trip) and the round-trips run
   * in parallel, so the page scales with `labels` rather than
   * `labels × assets-per-label`. The original-cased display label is taken
   * from the representative asset, not from the (possibly lowercased)
   * grouping key.
   *
   * @returns one {@link LabelEntry} per distinct primary label.
   */
  return async (): Promise<LabelEntry[]> => {
    const counts = await recordRepository.allPrimaryLabels();
    const samples = await Promise.all(
      counts.map((item) => recordRepository.latestAssetByLabel(item.label))
    );
    const entries: LabelEntry[] = [];
    for (const [i, item] of counts.entries()) {
      const sample = samples[i];
      if (!sample) continue;
      // Prefer the case-preserved label from the representative asset; fall
      // back to whatever the aggregate returned (may be lowercased by the
      // grouping key in some backends).
      const display = sample.primaryLabel || item.label;
      entries.push(new LabelEntry(display, item.count, sample.assetId));
    }
    return entries;
  };
};
