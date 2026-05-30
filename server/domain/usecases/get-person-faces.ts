//
// Copyright (c) 2026 Nathan Fiedler
//
import assert from 'node:assert';
import { type Face } from 'tanuki/server/domain/entities/face.ts';
import { type FaceStore } from 'tanuki/server/domain/repositories/face-store.ts';

export default ({ faceStore }: { faceStore: FaceStore }) => {
  assert.ok(faceStore, 'face store must be defined');
  /**
   * Return every face clustered under a person, for the cluster-management
   * view (multi-select reassignment, thumbnail pinning).
   *
   * @param personId - the person whose faces to return.
   */
  return async (personId: string): Promise<Face[]> => {
    return faceStore.facesForPerson(personId);
  };
};
