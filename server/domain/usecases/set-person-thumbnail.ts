//
// Copyright (c) 2026 Nathan Fiedler
//
import assert from 'node:assert';
import { type PersonSummary } from 'tanuki/server/domain/entities/face.ts';
import { type FaceStore } from 'tanuki/server/domain/repositories/face-store.ts';

export default ({ faceStore }: { faceStore: FaceStore }) => {
  assert.ok(faceStore, 'face store must be defined');
  /**
   * Pin a specific face (which must belong to the person) as its representative
   * thumbnail, and return the updated person.
   *
   * @param id - the person id.
   * @param faceId - the face to use as the thumbnail.
   */
  return async (
    id: string,
    faceId: string
  ): Promise<PersonSummary | null> => {
    await faceStore.setPersonThumbnail(id, faceId);
    return faceStore.getPersonSummary(id);
  };
};
