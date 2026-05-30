//
// Copyright (c) 2026 Nathan Fiedler
//
import { type SyntheticJob } from 'tanuki/server/domain/entities/face.ts';
import {
  SyntheticData,
  SyntheticStatus
} from 'tanuki/server/domain/entities/synthetic-data.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';
import { type SyntheticDetector } from 'tanuki/server/domain/services/synthetic-detector.ts';
import { type SyntheticJobProcessor } from 'tanuki/server/domain/services/synthetic-job-processor.ts';

/**
 * Worker-pool processor that runs the {@link SyntheticDetector} against a job's
 * asset and persists the result, marking the asset READY. Recoverable failures
 * (missing model, unreadable blob, inference error) throw so the pool can retry
 * and ultimately record FAILED.
 *
 * Phase 1 handles `labels` jobs; `faces` jobs throw until that pipeline lands.
 */
class DetectingSyntheticJobProcessor implements SyntheticJobProcessor {
  private recordRepository: RecordRepository;
  private detector: SyntheticDetector;

  constructor({
    recordRepository,
    syntheticDetector
  }: {
    recordRepository: RecordRepository;
    syntheticDetector: SyntheticDetector;
  }) {
    this.recordRepository = recordRepository;
    this.detector = syntheticDetector;
  }

  /** @inheritDoc */
  async process(job: SyntheticJob): Promise<void> {
    if (job.kind !== 'labels') {
      throw new Error(`unsupported synthetic job kind: ${job.kind}`);
    }
    const asset = await this.recordRepository.getAssetById(job.assetId);
    if (asset === null) {
      // asset was deleted before we got to it; nothing to do (don't retry)
      return;
    }
    const labels = await this.detector.detectLabels(asset);
    const data = new SyntheticData();
    data.labels = labels;
    data.primaryLabel = labels[0] ?? null;
    // Store null when nothing cleared the score floor, but still mark READY:
    // the asset was processed, it simply produced no labels.
    await this.recordRepository.setSynthetic(
      job.assetId,
      data.hasValues() ? data : null,
      SyntheticStatus.READY
    );
  }
}

export { DetectingSyntheticJobProcessor };
