//
// Copyright (c) 2026 Nathan Fiedler
//
import assert from 'node:assert';
import { type FaceStore } from 'tanuki/server/domain/repositories/face-store.ts';

export default ({ faceStore }: { faceStore: FaceStore }) => {
  assert.ok(faceStore, 'face store must be defined');
  /**
   * Hide every remaining unnamed person, removing the long tail of unnamed
   * face clusters from the People page in one shot. Named and already-hidden
   * people are left alone.
   *
   * @returns how many people were newly hidden.
   */
  return async (): Promise<number> => {
    return faceStore.hideUnnamedPeople();
  };
};
