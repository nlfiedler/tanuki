//
// Copyright (c) 2026 Nathan Fiedler
//
import assert from 'node:assert';
import { type PersonSummary } from 'tanuki/server/domain/entities/face.ts';
import { type FaceStore } from 'tanuki/server/domain/repositories/face-store.ts';

export default ({ faceStore }: { faceStore: FaceStore }) => {
  assert.ok(faceStore, 'face store must be defined');
  /**
   * List people for the People page, each enriched with face count and
   * representative face.
   *
   * @param includeHidden - when false, omit people flagged hidden.
   */
  return async (includeHidden: boolean): Promise<PersonSummary[]> => {
    return faceStore.listPeople(includeHidden);
  };
};
