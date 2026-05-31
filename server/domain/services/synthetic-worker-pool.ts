//
// Copyright (c) 2026 Nathan Fiedler
//
import { type SyntheticJob } from 'tanuki/server/domain/entities/face.ts';
import { SyntheticStatus } from 'tanuki/server/domain/entities/synthetic-data.ts';
import { type FaceStore } from 'tanuki/server/domain/repositories/face-store.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';
import { type SearchRepository } from 'tanuki/server/domain/repositories/search-repository.ts';
import { type SettingsRepository } from 'tanuki/server/domain/repositories/settings-repository.ts';
import { type SyntheticJobProcessor } from './synthetic-job-processor.ts';
import logger from 'tanuki/server/logger.ts';

/** Total runs allowed per job before it is abandoned (`up to 3 attempts`). */
const DEFAULT_MAX_ATTEMPTS = 3;
/** Exponential backoff between attempts, in milliseconds (1s, 4s, 16s). */
const DEFAULT_BACKOFF_MS = [1000, 4000, 16_000];
/** How long a worker waits before re-polling an empty queue. */
const DEFAULT_IDLE_MS = 1000;
/** Emit a progress log line once every this many completed jobs. */
const DEFAULT_LOG_EVERY = 100;

function realSleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Drains the face store's `synthetic_jobs` queue with a bounded pool of
 * concurrent workers. Each worker claims the highest-priority, oldest job
 * (so live imports preempt backfill without separate queues), hands it to the
 * {@link SyntheticJobProcessor}, and on failure retries with exponential
 * backoff up to a fixed attempt budget — after which the asset is recorded as
 * FAILED.
 *
 * Pool size comes from `SYNTHETIC_CONCURRENCY` (default 2). The retry/backoff
 * knobs and a `sleep` seam are constructor-overridable so tests run without
 * real delays.
 */
class SyntheticWorkerPool {
  private faceStore: FaceStore;
  private recordRepository: RecordRepository;
  private searchRepository: SearchRepository;
  private processor: SyntheticJobProcessor;
  private concurrency: number;
  private maxAttempts: number;
  private backoffMs: number[];
  private idleMs: number;
  private logEvery: number;
  private sleep: (ms: number) => Promise<void>;
  private running = false;
  private workers: Promise<void>[] = [];
  /** Jobs completed (success or terminal failure) since the pool last started. */
  private processed = 0;

  /**
   * Constructed by awilix from the container cradle (first arg). Tests pass a
   * second `overrides` arg to inject a fake `sleep` and tweak the retry/idle
   * timing without delays — kept out of the cradle destructure so awilix
   * doesn't try to resolve them as DI deps and throw at startup.
   */
  constructor(
    {
      faceStore,
      recordRepository,
      searchRepository,
      syntheticJobProcessor,
      settingsRepository
    }: {
      faceStore: FaceStore;
      recordRepository: RecordRepository;
      searchRepository: SearchRepository;
      syntheticJobProcessor: SyntheticJobProcessor;
      settingsRepository: SettingsRepository;
    },
    overrides: {
      sleep?: (ms: number) => Promise<void>;
      concurrency?: number;
      maxAttempts?: number;
      backoffMs?: number[];
      idleMs?: number;
      logEvery?: number;
    } = {}
  ) {
    this.faceStore = faceStore;
    this.recordRepository = recordRepository;
    this.searchRepository = searchRepository;
    this.processor = syntheticJobProcessor;
    this.concurrency = Math.max(
      1,
      overrides.concurrency ??
        settingsRepository.getInt('SYNTHETIC_CONCURRENCY', 2)
    );
    this.maxAttempts = overrides.maxAttempts ?? DEFAULT_MAX_ATTEMPTS;
    this.backoffMs = overrides.backoffMs ?? DEFAULT_BACKOFF_MS;
    this.idleMs = overrides.idleMs ?? DEFAULT_IDLE_MS;
    this.logEvery = Math.max(
      1,
      overrides.logEvery ??
        settingsRepository.getInt('SYNTHETIC_LOG_EVERY', DEFAULT_LOG_EVERY)
    );
    this.sleep = overrides.sleep ?? realSleep;
  }

  /** Start the worker loops. Idempotent: a second call while running is a no-op. */
  start(): void {
    if (this.running) return;
    this.running = true;
    this.processed = 0;
    this.workers = [];
    for (let i = 0; i < this.concurrency; i++) {
      this.workers.push(this.runLoop());
    }
    logger.info(`synthetic worker pool started (concurrency ${this.concurrency})`);
  }

  /** Stop accepting work and wait for in-flight loops to finish. */
  async stop(): Promise<void> {
    this.running = false;
    await Promise.allSettled(this.workers);
    this.workers = [];
  }

  private async runLoop(): Promise<void> {
    while (this.running) {
      let job: SyntheticJob | null = null;
      try {
        job = await this.faceStore.claimNextJob();
      } catch (error: any) {
        logger.error('synthetic worker: failed to claim job:', error);
      }
      if (job === null) {
        // queue empty (or unreadable): wait before polling again
        await this.sleep(this.idleMs);
        continue;
      }
      await this.runJob(job);
    }
  }

  private async runJob(job: SyntheticJob): Promise<void> {
    try {
      await this.processor.process(job);
      // Success: the processor persisted results and set status READY. Drop
      // the search cache so a query made before classification returns the
      // newly labelled asset.
      await this.invalidateSearchCache();
      await this.recordCompletion();
    } catch (error: any) {
      const message = error instanceof Error ? error.message : String(error);
      const attempts = job.attempts + 1;
      if (attempts >= this.maxAttempts) {
        await this.markFailed(job, attempts, message);
        await this.recordCompletion();
        return;
      }
      // Requeue immediately with a not_before delay equal to the backoff
      // window. The row is visible to hasPendingJob during that window so
      // backfill/retry won't enqueue a duplicate, and claimNextJob filters on
      // not_before so the backoff is still observed by every worker.
      const index = Math.min(attempts - 1, this.backoffMs.length - 1);
      const delaySeconds = Math.ceil(
        (this.backoffMs[index] ?? DEFAULT_IDLE_MS) / 1000
      );
      try {
        await this.faceStore.requeueJob(job, message, delaySeconds);
      } catch (requeueError: any) {
        // Falling back to FAILED is safer than logging-and-dropping: the
        // claimed row is gone from the queue, and without this fallback the
        // asset would be stuck at PENDING forever, invisible to retry.
        logger.error(
          'synthetic worker: requeueJob failed, marking asset FAILED:',
          requeueError
        );
        await this.markFailed(job, attempts, message);
        await this.recordCompletion();
      }
    }
  }

  /**
   * Tally a finished job (success or terminal failure — not a retry requeue)
   * and emit a progress line every `logEvery` completions, including the
   * jobs still queued so an operator can gauge how far a backfill has to go.
   */
  private async recordCompletion(): Promise<void> {
    this.processed++;
    if (this.processed % this.logEvery !== 0) return;
    try {
      const remaining = await this.faceStore.pendingJobCount();
      logger.info(
        `synthetic worker pool: ${this.processed} jobs processed this run, ~${remaining} still queued`
      );
    } catch {
      // A failed count read must not interrupt processing; log progress anyway.
      logger.info(
        `synthetic worker pool: ${this.processed} jobs processed this run`
      );
    }
  }

  private async markFailed(
    job: SyntheticJob,
    attempts: number,
    message: string
  ): Promise<void> {
    logger.warn(
      `synthetic job ${job.id} (${job.kind}) for asset ${job.assetId} failed after ${attempts} attempts: ${message}`
    );
    try {
      // Record FAILED against the kind that failed: labels status lives on the
      // asset record, faces status lives in the face store. The GraphQL
      // syntheticStatus is the worse of the two, so this surfaces correctly
      // even when the other kind succeeded.
      if (job.kind === 'faces') {
        await this.faceStore.setFacesStatus(
          job.assetId,
          SyntheticStatus.FAILED
        );
      } else {
        await this.recordRepository.setSynthetic(
          job.assetId,
          null,
          SyntheticStatus.FAILED
        );
      }
      await this.invalidateSearchCache();
    } catch (markError: any) {
      logger.error('synthetic worker: failed to mark asset FAILED:', markError);
    }
  }

  private async invalidateSearchCache(): Promise<void> {
    try {
      await this.searchRepository.clear();
    } catch (error: any) {
      logger.warn('synthetic worker: search cache invalidation failed:', error);
    }
  }
}

export { SyntheticWorkerPool };
