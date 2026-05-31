//
// Copyright (c) 2026 Nathan Fiedler
//
import { beforeEach, describe, expect, test } from 'bun:test';
// prepare the test environment as early as possible
import 'tanuki/test/env.ts';
import { Face } from 'tanuki/server/domain/entities/face.ts';
import { SyntheticStatus } from 'tanuki/server/domain/entities/synthetic-data.ts';
import { EnvSettingsRepository } from 'tanuki/server/data/repositories/env-settings-repository.ts';
import { SqliteFaceStore } from 'tanuki/server/data/repositories/sqlite-face-store.ts';

/** Build an L2-normalized embedding from raw values (cosine = dot for units). */
function unit(values: number[]): Float32Array {
  const v = Float32Array.from(values);
  const norm = Math.hypot(...values) || 1;
  for (let i = 0; i < v.length; i++) v[i]! /= norm;
  return v;
}

let faceSeq = 0;
/** Construct a Face entity with sensible defaults for the field under test. */
function makeFace(opts: {
  assetId: string;
  personId?: string | null;
  bbox?: [number, number, number, number];
  embedding?: Float32Array;
  thumbnail?: Uint8Array;
  detectorScore?: number | null;
  modelVersion?: string;
  id?: string;
}): Face {
  return new Face(
    opts.id ?? `face-${++faceSeq}`,
    opts.assetId,
    opts.bbox ?? [0, 0, 10, 10],
    opts.embedding ?? unit([1, 0, 0]),
    opts.thumbnail ?? Uint8Array.from([1, 2, 3]),
    opts.modelVersion ?? 'mobilefacenet-v1',
    opts.personId ?? null,
    opts.detectorScore ?? 0.9
  );
}

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

    test('facesStatusCount tallies assets per terminal status', async function () {
      expect(await sut.facesStatusCount(SyntheticStatus.READY)).toEqual(0);
      await sut.setFacesStatus('asset-a', SyntheticStatus.READY);
      await sut.setFacesStatus('asset-b', SyntheticStatus.READY);
      await sut.setFacesStatus('asset-c', SyntheticStatus.FAILED);
      expect(await sut.facesStatusCount(SyntheticStatus.READY)).toEqual(2);
      expect(await sut.facesStatusCount(SyntheticStatus.FAILED)).toEqual(1);
      // PENDING clears the row, so it stops counting toward any terminal status
      await sut.setFacesStatus('asset-a', SyntheticStatus.PENDING);
      expect(await sut.facesStatusCount(SyntheticStatus.READY)).toEqual(1);
    });
  });

  describe('faces and people', function () {
    test('createPerson yields a distinct, unnamed, visible person', async function () {
      const a = await sut.createPerson();
      const b = await sut.createPerson();
      expect(a.id).not.toEqual(b.id);
      expect(a.name).toBeNull();
      expect(a.hidden).toBeFalse();
      const summary = await sut.getPersonSummary(a.id);
      expect(summary!.faceCount).toEqual(0);
      expect(summary!.representativeFaceId).toBeNull();
    });

    test('inserted faces drive face count and representative selection', async function () {
      const person = await sut.createPerson();
      // small face, large face: representative should be the larger one
      await sut.insertFace(
        makeFace({ id: 'small', assetId: 'asset-1', personId: person.id, bbox: [0, 0, 10, 10] })
      );
      await sut.insertFace(
        makeFace({ id: 'big', assetId: 'asset-1', personId: person.id, bbox: [0, 0, 50, 50] })
      );
      const summary = await sut.getPersonSummary(person.id);
      expect(summary!.faceCount).toEqual(2);
      expect(summary!.representativeFaceId).toEqual('big');
    });

    test('representative ties broken by detector score', async function () {
      const person = await sut.createPerson();
      await sut.insertFace(
        makeFace({ id: 'lo', assetId: 'a', personId: person.id, bbox: [0, 0, 20, 20], detectorScore: 0.6 })
      );
      await sut.insertFace(
        makeFace({ id: 'hi', assetId: 'a', personId: person.id, bbox: [0, 0, 20, 20], detectorScore: 0.95 })
      );
      const summary = await sut.getPersonSummary(person.id);
      expect(summary!.representativeFaceId).toEqual('hi');
    });

    test('setPersonThumbnail pins a face, but only one that belongs', async function () {
      const person = await sut.createPerson();
      await sut.insertFace(
        makeFace({ id: 'big', assetId: 'a', personId: person.id, bbox: [0, 0, 50, 50] })
      );
      await sut.insertFace(
        makeFace({ id: 'small', assetId: 'a', personId: person.id, bbox: [0, 0, 10, 10] })
      );
      await sut.setPersonThumbnail(person.id, 'small');
      const summary = await sut.getPersonSummary(person.id);
      expect(summary!.representativeFaceId).toEqual('small');
      // a face from another person cannot be pinned
      const other = await sut.createPerson();
      await sut.insertFace(
        makeFace({ id: 'foreign', assetId: 'b', personId: other.id })
      );
      expect(sut.setPersonThumbnail(person.id, 'foreign')).rejects.toThrow();
    });

    test('fetchPeopleByAssetIds batches and groups by asset', async function () {
      const alice = await sut.createPerson();
      const bob = await sut.createPerson();
      await sut.insertFace(makeFace({ assetId: 'asset-1', personId: alice.id }));
      await sut.insertFace(makeFace({ assetId: 'asset-1', personId: bob.id }));
      await sut.insertFace(makeFace({ assetId: 'asset-2', personId: alice.id }));
      // an unclustered face must not surface as a person
      await sut.insertFace(makeFace({ assetId: 'asset-2', personId: null }));

      const map = await sut.fetchPeopleByAssetIds(['asset-1', 'asset-2', 'asset-3']);
      expect(map.get('asset-1')!.map((s) => s.person.id).sort()).toEqual(
        [alice.id, bob.id].sort()
      );
      expect(map.get('asset-2')!.map((s) => s.person.id)).toEqual([alice.id]);
      expect(map.get('asset-3')).toEqual([]);
    });

    test('assetIdsByPerson pages distinct assets, newest first', async function () {
      const person = await sut.createPerson();
      await sut.insertFace(makeFace({ assetId: 'asset-1', personId: person.id }));
      await sut.insertFace(makeFace({ assetId: 'asset-2', personId: person.id }));
      // a second face on asset-2 should not make it appear twice
      await sut.insertFace(makeFace({ assetId: 'asset-2', personId: person.id }));
      await sut.insertFace(makeFace({ assetId: 'asset-3', personId: person.id }));

      const page = await sut.assetIdsByPerson(person.id, 0, 2);
      expect(page.total).toEqual(3);
      // ordered by most-recently-inserted face: asset-3, then asset-2
      expect(page.ids).toEqual(['asset-3', 'asset-2']);
      const next = await sut.assetIdsByPerson(person.id, 2, 2);
      expect(next.ids).toEqual(['asset-1']);
    });

    test('nearestPerson matches within a model version only', async function () {
      const alice = await sut.createPerson();
      await sut.insertFace(
        makeFace({ assetId: 'a', personId: alice.id, embedding: unit([1, 0, 0]) })
      );
      // near-identical embedding finds Alice with a high cosine score
      const hit = await sut.nearestPerson(unit([0.99, 0.01, 0]), 'mobilefacenet-v1');
      expect(hit!.personId).toEqual(alice.id);
      expect(hit!.score).toBeGreaterThan(0.9);
      // a different model version shares no comparable faces
      expect(await sut.nearestPerson(unit([1, 0, 0]), 'mobilefacenet-v2')).toBeNull();
      // empty store yields null
    });

    test('mergePeople reassigns faces and deletes the source', async function () {
      const alice = await sut.createPerson();
      const bob = await sut.createPerson();
      await sut.insertFace(makeFace({ assetId: 'a', personId: alice.id }));
      await sut.insertFace(makeFace({ assetId: 'b', personId: bob.id }));

      await sut.mergePeople(bob.id, alice.id);
      expect(await sut.getPersonSummary(bob.id)).toBeNull();
      const summary = await sut.getPersonSummary(alice.id);
      expect(summary!.faceCount).toEqual(2);
    });

    test('reassignFaces splits into a new person and prunes empty source', async function () {
      const alice = await sut.createPerson();
      await sut.insertFace(makeFace({ id: 'f1', assetId: 'a', personId: alice.id }));
      await sut.insertFace(makeFace({ id: 'f2', assetId: 'b', personId: alice.id }));

      // move both faces to a brand-new person -> Alice becomes empty -> deleted
      const newId = await sut.reassignFaces(['f1', 'f2'], null);
      expect(newId).not.toEqual(alice.id);
      expect(await sut.getPersonSummary(alice.id)).toBeNull();
      expect((await sut.getPersonSummary(newId))!.faceCount).toEqual(2);
    });

    test('reassignFaces to an existing person keeps the source when faces remain', async function () {
      const alice = await sut.createPerson();
      const bob = await sut.createPerson();
      await sut.insertFace(makeFace({ id: 'f1', assetId: 'a', personId: alice.id }));
      await sut.insertFace(makeFace({ id: 'f2', assetId: 'b', personId: alice.id }));

      await sut.reassignFaces(['f1'], bob.id);
      expect((await sut.getPersonSummary(alice.id))!.faceCount).toEqual(1);
      expect((await sut.getPersonSummary(bob.id))!.faceCount).toEqual(1);
    });

    test('reassignFaces rejects an empty face list', async function () {
      expect(sut.reassignFaces([], null)).rejects.toThrow();
    });

    test('renamePerson normalizes blank names to null', async function () {
      const person = await sut.createPerson();
      await sut.renamePerson(person.id, '  Alice  ');
      expect((await sut.getPersonSummary(person.id))!.person.name).toEqual('Alice');
      await sut.renamePerson(person.id, '   ');
      expect((await sut.getPersonSummary(person.id))!.person.name).toBeNull();
    });

    test('personIdsByName matches case-insensitively, ignoring unnamed', async function () {
      const alice1 = await sut.createPerson();
      const alice2 = await sut.createPerson();
      const bob = await sut.createPerson();
      const unnamed = await sut.createPerson();
      await sut.renamePerson(alice1.id, 'Alice');
      await sut.renamePerson(alice2.id, 'alice'); // same name, different casing
      await sut.renamePerson(bob.id, 'Bob');

      const aliceMatches = await sut.personIdsByName('ALICE');
      expect(aliceMatches.sort()).toEqual([alice1.id, alice2.id].sort());
      expect(await sut.personIdsByName('bob')).toEqual([bob.id]);
      expect(await sut.personIdsByName('nobody')).toEqual([]);
      // the unnamed person is never matched
      const emptyMatches = await sut.personIdsByName('');
      expect(emptyMatches).not.toContain(unnamed.id);
    });

    test('hidePerson flips visibility and listPeople filters', async function () {
      const visible = await sut.createPerson();
      const hidden = await sut.createPerson();
      await sut.hidePerson(hidden.id, true);
      const shown = await sut.listPeople(false);
      expect(shown.map((s) => s.person.id)).toEqual([visible.id]);
      const all = await sut.listPeople(true);
      expect(all.map((s) => s.person.id).sort()).toEqual(
        [visible.id, hidden.id].sort()
      );
    });

    test('listPeople orders by descending face count', async function () {
      const lonely = await sut.createPerson();
      await sut.insertFace(makeFace({ assetId: 'a', personId: lonely.id }));
      const popular = await sut.createPerson();
      await sut.insertFace(makeFace({ assetId: 'b', personId: popular.id }));
      await sut.insertFace(makeFace({ assetId: 'c', personId: popular.id }));
      await sut.insertFace(makeFace({ assetId: 'd', personId: popular.id }));
      const middling = await sut.createPerson();
      await sut.insertFace(makeFace({ assetId: 'e', personId: middling.id }));
      await sut.insertFace(makeFace({ assetId: 'f', personId: middling.id }));

      const people = await sut.listPeople(false);
      // most-photographed first, regardless of creation order
      expect(people.map((s) => s.person.id)).toEqual([
        popular.id,
        middling.id,
        lonely.id
      ]);
      expect(people.map((s) => s.faceCount)).toEqual([3, 2, 1]);
    });

    test('hideUnnamedPeople hides only visible, unnamed people', async function () {
      const named = await sut.createPerson();
      await sut.renamePerson(named.id, 'Alice');
      const unnamed1 = await sut.createPerson();
      const unnamed2 = await sut.createPerson();
      const alreadyHidden = await sut.createPerson();
      await sut.hidePerson(alreadyHidden.id, true);

      // only the two visible unnamed people are newly hidden
      expect(await sut.hideUnnamedPeople()).toEqual(2);
      const visible = await sut.listPeople(false);
      expect(visible.map((s) => s.person.id)).toEqual([named.id]);
      // running again is a no-op now that nothing visible is unnamed
      expect(await sut.hideUnnamedPeople()).toEqual(0);
      // the named person is still untouched and visible
      expect((await sut.getPersonSummary(named.id))!.person.hidden).toBeFalse();
      // the previously-unnamed clusters are now hidden
      for (const id of [unnamed1.id, unnamed2.id]) {
        expect((await sut.getPersonSummary(id))!.person.hidden).toBeTrue();
      }
    });

    test('facesForPerson and faceThumbnail return stored crops', async function () {
      const person = await sut.createPerson();
      await sut.insertFace(
        makeFace({ id: 'f1', assetId: 'a', personId: person.id, thumbnail: Uint8Array.from([9, 8, 7]) })
      );
      const faces = await sut.facesForPerson(person.id);
      expect(faces).toHaveLength(1);
      expect(faces[0]!.assetId).toEqual('a');
      expect([...faces[0]!.embedding]).toEqual([...unit([1, 0, 0])]);
      const thumb = await sut.faceThumbnail('f1');
      expect([...thumb!]).toEqual([9, 8, 7]);
      expect(await sut.faceThumbnail('missing')).toBeNull();
    });

    test('deleteByAssetId removes faces and cascades empty people', async function () {
      const alice = await sut.createPerson();
      const bob = await sut.createPerson();
      await sut.insertFace(makeFace({ assetId: 'asset-1', personId: alice.id }));
      await sut.insertFace(makeFace({ assetId: 'asset-2', personId: bob.id }));

      await sut.deleteByAssetId('asset-1');
      // Alice had only the asset-1 face -> cascade-deleted; Bob survives
      expect(await sut.getPersonSummary(alice.id)).toBeNull();
      expect(await sut.getPersonSummary(bob.id)).not.toBeNull();
      expect(await sut.allFaceAssetIds()).toEqual(['asset-2']);
    });
  });
});
