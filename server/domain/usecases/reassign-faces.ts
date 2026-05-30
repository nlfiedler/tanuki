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
   * Reassign faces to a person (splitting a cluster); a null `personId` creates
   * a new person. Returns the destination person. Clears the search cache
   * because `person:` query results depend on cluster membership.
   *
   * @param faceIds - the faces to move.
   * @param personId - destination person, or null to create one.
   */
  return async (
    faceIds: string[],
    personId: string | null
  ): Promise<PersonSummary | null> => {
    const destination = await faceStore.reassignFaces(faceIds, personId);
    await searchRepository.clear();
    return faceStore.getPersonSummary(destination);
  };
};
