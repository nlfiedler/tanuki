//
// Copyright (c) 2026 Nathan Fiedler
//
import assert from 'node:assert';
import { type PersonSummary } from 'tanuki/server/domain/entities/face.ts';
import { type FaceStore } from 'tanuki/server/domain/repositories/face-store.ts';

export default ({ faceStore }: { faceStore: FaceStore }) => {
  assert.ok(faceStore, 'face store must be defined');
  /**
   * Hide or unhide a person (excluding it from the People page without removing
   * its faces), and return the updated person.
   *
   * @param id - the person id.
   * @param hidden - the new hidden flag.
   */
  return async (
    id: string,
    hidden: boolean
  ): Promise<PersonSummary | null> => {
    await faceStore.hidePerson(id, hidden);
    return faceStore.getPersonSummary(id);
  };
};
