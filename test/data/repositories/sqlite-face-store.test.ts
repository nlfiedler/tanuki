//
// Copyright (c) 2026 Nathan Fiedler
//
import { beforeEach, describe, expect, test } from 'bun:test';
// prepare the test environment as early as possible
import 'tanuki/test/env.ts';
import { EnvSettingsRepository } from 'tanuki/server/data/repositories/env-settings-repository.ts';
import { SqliteFaceStore } from 'tanuki/server/data/repositories/sqlite-face-store.ts';

describe('SqliteFaceStore', function () {
  const settingsRepository = new EnvSettingsRepository();
  const sut = new SqliteFaceStore({ settingsRepository });

  beforeEach(async function () {
    await sut.destroyAndCreate();
  });

  describe('synthetic_jobs queue', function () {
    test('claimNextJob returns null on an empty queue', async function () {
      expect(await sut.claimNextJob()).toBeNull();
      expect(await sut.pendingJobCount()).toEqual(0);
    });

    test('enqueueJob assigns ids and counts pending jobs', async function () {
      const first = await sut.enqueueJob('asset-a', 'labels');
      const second = await sut.enqueueJob('asset-b', 'faces', 10);
      expect(second).toBeGreaterThan(first);
      expect(await sut.pendingJobCount()).toEqual(2);
      expect(await sut.pendingJobCount('labels')).toEqual(1);
      expect(await sut.pendingJobCount('faces')).toEqual(1);
    });

    test('claimNextJob drains highest priority first, then oldest', async function () {
      // enqueue out of priority order to prove ordering is by priority, not id
      await sut.enqueueJob('low-old', 'labels', 0);
      await sut.enqueueJob('low-new', 'labels', 0);
      await sut.enqueueJob('high', 'faces', 10);

      const a = await sut.claimNextJob();
      expect(a!.assetId).toEqual('high');
      const b = await sut.claimNextJob();
      expect(b!.assetId).toEqual('low-old');
      const c = await sut.claimNextJob();
      expect(c!.assetId).toEqual('low-new');
      expect(await sut.claimNextJob()).toBeNull();
    });

    test('claiming a job removes it from the queue', async function () {
      await sut.enqueueJob('asset-a', 'labels');
      expect(await sut.pendingJobCount()).toEqual(1);
      const job = await sut.claimNextJob();
      expect(job!.attempts).toEqual(0);
      expect(job!.lastError).toBeNull();
      expect(await sut.pendingJobCount()).toEqual(0);
    });

    test('requeueJob re-enqueues with incremented attempts and the error', async function () {
      await sut.enqueueJob('asset-a', 'faces', 5);

      // first failure -> requeued, attempts = 1
      const j1 = await sut.claimNextJob();
      expect(await sut.requeueJob(j1!, 'boom 1')).toEqual(1);
      expect(await sut.pendingJobCount()).toEqual(1);

      // the requeued job preserves priority and carries the error forward
      const j2 = await sut.claimNextJob();
      expect(j2!.assetId).toEqual('asset-a');
      expect(j2!.priority).toEqual(5);
      expect(j2!.attempts).toEqual(1);
      expect(j2!.lastError).toEqual('boom 1');

      // second failure -> requeued, attempts = 2
      expect(await sut.requeueJob(j2!, 'boom 2')).toEqual(2);
      const j3 = await sut.claimNextJob();
      expect(j3!.attempts).toEqual(2);
      expect(j3!.lastError).toEqual('boom 2');

      // a job is only back on the queue when explicitly requeued
      expect(await sut.pendingJobCount()).toEqual(0);
    });
  });

  describe('Phase 2 face/person methods', function () {
    test('are declared but not yet implemented', async function () {
      expect(sut.fetchPeopleByAssetIds(['a'])).rejects.toThrow('Phase 2');
      expect(sut.assetIdsByPerson('p', 0, 10)).rejects.toThrow('Phase 2');
      expect(sut.deleteByAssetId('a')).rejects.toThrow('Phase 2');
    });
  });
});
