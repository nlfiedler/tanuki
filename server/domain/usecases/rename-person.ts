//
// Copyright (c) 2026 Nathan Fiedler
//
import assert from 'node:assert';
import { type PersonSummary } from 'tanuki/server/domain/entities/face.ts';
import { type FaceStore } from 'tanuki/server/domain/repositories/face-store.ts';

export default ({ faceStore }: { faceStore: FaceStore }) => {
  assert.ok(faceStore, 'face store must be defined');
  /**
   * Set or clear a person's user-assigned name and return the updated person.
   *
   * @param id - the person id.
   * @param name - the new name, or null to clear.
   */
  return async (
    id: string,
    name: string | null
  ): Promise<PersonSummary | null> => {
    await faceStore.renamePerson(id, name);
    return faceStore.getPersonSummary(id);
  };
};
