//
// Copyright (c) 2026 Nathan Fiedler
//
import { afterEach, beforeEach, describe, expect, test } from 'bun:test';
// prepare the test environment as early as possible
import 'tanuki/test/env.ts';
import { SyntheticData, SyntheticStatus } from 'tanuki/server/domain/entities/synthetic-data.ts';
import { SyntheticJob } from 'tanuki/server/domain/entities/face.ts';
import { EnvSettingsRepository } from 'tanuki/server/data/repositories/env-settings-repository.ts';
import { SqliteFaceStore } from 'tanuki/server/data/repositories/sqlite-face-store.ts';
import { SyntheticWorkerPool } from 'tanuki/server/domain/services/synthetic-worker-pool.ts';

/** Wait until `predicate` is true, polling briefly; throws on timeout. */
async function until(
  predicate: () => boolean | Promise<boolean>,
  timeoutMs = 2000
): Promise<void> {
  const start = Date.now();
  while (!(await predicate())) {
    if (Date.now() - start > timeoutMs) {
      throw new Error('until(): condition not met before timeout');
    }
    await new Promise((resolve) => setTimeout(resolve, 5));
  }
}

/** Captures setSynthetic calls; everything else is unused by the pool. */
function makeRecordRepository(): any {
  const failed: string[] = [];
  return {
    failed,
    async setSynthetic(
      assetId: string,
      _data: SyntheticData | null,
      status: SyntheticStatus
    ): Promise<void> {
      if (status === SyntheticStatus.FAILED) failed.push(assetId);
    }
  };
}

/** Captures clear() calls so tests can assert the search cache was invalidated. */
function makeSearchRepository(): any {
  let cleared = 0;
  return {
    get cleared() {
      return cleared;
    },
    async clear(): Promise<void> {
      cleared++;
    }
  };
}

describe('SyntheticWorkerPool', function () {
  const settingsRepository = new EnvSettingsRepository();
  // use a dedicated database file so this suite never collides with the
  // face-store suite's WAL sidecars when both run in one process
  settingsRepository.set('FACE_STORE_PATH', 'tmp/test/faces-pool');
  const faceStore = new SqliteFaceStore({ settingsRepository });
  let recordRepository: any;
  let searchRepository: any;
  let pool: SyntheticWorkerPool | null = null;
  const sleeps: number[] = [];
  // near-instant sleep that records the requested delay but still yields to
  // the macrotask queue, so idle worker loops don't starve timers/IO
  const sleep = async (ms: number): Promise<void> => {
    sleeps.push(ms);
    await new Promise((resolve) => setTimeout(resolve, 0));
  };

  beforeEach(async function () {
    await faceStore.destroyAndCreate();
    recordRepository = makeRecordRepository();
    searchRepository = makeSearchRepository();
    sleeps.length = 0;
  });

  // stop any pool a test started, even if it failed before its own stop(),
  // so a runaway loop can't hammer the database during the next beforeEach
  afterEach(async function () {
    if (pool) {
      await pool.stop();
      pool = null;
    }
  });

  test('drains every queued job exactly once', async function () {
    const seen: string[] = [];
    const processor = {
      async process(job: SyntheticJob): Promise<void> {
        seen.push(job.assetId);
      }
    };
    await faceStore.enqueueJob('a', 'labels');
    await faceStore.enqueueJob('b', 'labels');
    await faceStore.enqueueJob('c', 'faces', 10);

    pool = new SyntheticWorkerPool(
      {
        faceStore,
        recordRepository,
        searchRepository,
        syntheticJobProcessor: processor,
        settingsRepository
      },
      { sleep, concurrency: 2, idleMs: 0 }
    );
    pool.start();
    await until(async () => (await faceStore.pendingJobCount()) === 0);
    await pool.stop();

    expect(seen.slice().sort()).toEqual(['a', 'b', 'c']);
    expect(await faceStore.pendingJobCount()).toEqual(0);
    expect(recordRepository.failed).toEqual([]);
  });

  test('never runs more jobs concurrently than the pool size', async function () {
    let active = 0;
    let peak = 0;
    const processor = {
      async process(_job: SyntheticJob): Promise<void> {
        active++;
        peak = Math.max(peak, active);
        // hold the slot long enough that a second worker overlaps
        await new Promise((resolve) => setTimeout(resolve, 15));
        active--;
      }
    };
    for (const id of ['a', 'b', 'c', 'd', 'e']) {
      await faceStore.enqueueJob(id, 'labels');
    }

    pool = new SyntheticWorkerPool(
      {
        faceStore,
        recordRepository,
        searchRepository,
        syntheticJobProcessor: processor,
        settingsRepository
      },
      { sleep, concurrency: 2, idleMs: 0 }
    );
    pool.start();
    await until(
      async () => (await faceStore.pendingJobCount()) === 0 && active === 0
    );
    await pool.stop();

    // two workers, five jobs: they must overlap, but never exceed the pool size
    expect(peak).toEqual(2);
  });

  test('retries up to maxAttempts then marks the asset FAILED', async function () {
    let calls = 0;
    const processor = {
      async process(_job: SyntheticJob): Promise<void> {
        calls++;
        throw new Error('detector unavailable');
      }
    };
    await faceStore.enqueueJob('doomed', 'faces', 5);

    pool = new SyntheticWorkerPool(
      {
        faceStore,
        recordRepository,
        searchRepository,
        syntheticJobProcessor: processor,
        settingsRepository
      },
      {
        sleep,
        concurrency: 1,
        maxAttempts: 3,
        // zero backoff so the queue's not_before filter doesn't stall the test
        backoffMs: [0, 0, 0],
        idleMs: 0
      }
    );
    pool.start();
    // a faces failure is recorded in the face store (not on the asset record)
    const facesStatus = async (): Promise<SyntheticStatus | undefined> => {
      const map = await faceStore.fetchFacesStatus(['doomed']);
      return map.get('doomed');
    };
    await until(async () => (await facesStatus()) === SyntheticStatus.FAILED);
    await pool.stop();

    // three runs total (initial + two retries), then give up
    expect(calls).toEqual(3);
    expect(await facesStatus()).toEqual(SyntheticStatus.FAILED);
    // labels status untouched: kind-aware failure handling
    expect(recordRepository.failed).toEqual([]);
    // queue is empty: a FAILED job is not left lingering
    expect(await faceStore.pendingJobCount()).toEqual(0);
    // the search cache was cleared on terminal failure (and would be on success)
    expect(searchRepository.cleared).toBeGreaterThanOrEqual(1);
  });

  test('requeueJob keeps the job visible to hasPendingJob during backoff', async function () {
    // Use a tiny backoff window — long enough to observe, short enough to wait
    // out. Confirms the new architecture: a failed job is back in the queue
    // immediately (so backfill sees it via hasPendingJob), but is not yet
    // claimable until not_before elapses.
    let calls = 0;
    const processor = {
      async process(_job: SyntheticJob): Promise<void> {
        calls++;
        if (calls === 1) throw new Error('first try fails');
      }
    };
    await faceStore.enqueueJob('backoff-test', 'labels');

    pool = new SyntheticWorkerPool(
      {
        faceStore,
        recordRepository,
        searchRepository,
        syntheticJobProcessor: processor,
        settingsRepository
      },
      { sleep, concurrency: 1, backoffMs: [1], idleMs: 0 }
    );
    pool.start();
    // wait for the first failure to land back in the queue
    await until(async () => (await faceStore.pendingJobCount()) === 1);
    // hasPendingJob sees it — duplicate enqueue is blocked
    expect(await faceStore.hasPendingJob('backoff-test', 'labels')).toBe(true);
    await until(() => calls >= 2);
    await pool.stop();

    expect(calls).toEqual(2);
    expect(recordRepository.failed).toEqual([]);
  });

  test('a transient failure is retried and then succeeds', async function () {
    let calls = 0;
    const processor = {
      async process(_job: SyntheticJob): Promise<void> {
        calls++;
        if (calls === 1) throw new Error('flaky first attempt');
      }
    };
    await faceStore.enqueueJob('flaky', 'labels');

    pool = new SyntheticWorkerPool(
      {
        faceStore,
        recordRepository,
        searchRepository,
        syntheticJobProcessor: processor,
        settingsRepository
      },
      { sleep, concurrency: 1, backoffMs: [0], idleMs: 0 }
    );
    pool.start();
    await until(async () => (await faceStore.pendingJobCount()) === 0 && calls >= 2);
    await pool.stop();

    expect(calls).toEqual(2);
    expect(recordRepository.failed).toEqual([]);
  });

  test('stop() resolves and the loop processes nothing further', async function () {
    const processor = {
      async process(_job: SyntheticJob): Promise<void> {}
    };
    pool = new SyntheticWorkerPool(
      {
        faceStore,
        recordRepository,
        searchRepository,
        syntheticJobProcessor: processor,
        settingsRepository
      },
      { sleep, concurrency: 2, idleMs: 0 }
    );
    pool.start();
    await pool.stop();
    // enqueue after stop; nothing should drain it
    await faceStore.enqueueJob('late', 'labels');
    await new Promise((resolve) => setTimeout(resolve, 30));
    expect(await faceStore.pendingJobCount()).toEqual(1);
  });
});
