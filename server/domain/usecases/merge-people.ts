//
// Copyright (c) 2026 Nathan Fiedler
//
import assert from 'node:assert';
import { type PersonSummary } from 'tanuki/server/domain/entities/face.ts';
import { type FaceStore } from 'tanuki/server/domain/repositories/face-store.ts';
import { type SearchRepository } from 'tanuki/server/domain/repositories/search-repository.ts';

export default ({
  faceStore,
  searchRepository
}: {
  faceStore: FaceStore;
  searchRepository: SearchRepository;
}) => {
  assert.ok(faceStore, 'face store must be defined');
  assert.ok(searchRepository, 'search repository must be defined');
  /**
   * Merge the source person into the target (reassigning all of the source's
   * faces, then deleting the source), and return the target. Clears the search
   * cache because `person:` query results depend on cluster membership.
   *
   * @param sourceId - person to merge from (deleted afterwards).
   * @param targetId - person to merge into (survives).
   */
  return async (
    sourceId: string,
    targetId: string
  ): Promise<PersonSummary | null> => {
    await faceStore.mergePeople(sourceId, targetId);
    await searchRepository.clear();
    return faceStore.getPersonSummary(targetId);
  };
};
