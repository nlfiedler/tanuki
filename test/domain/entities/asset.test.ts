//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, test } from 'bun:test';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';

describe('Asset entity', function () {
  test('should return the most accurate date', function () {
    // arrange
    const asset = new Asset('abc123');
    asset.importDate = new Date(2018, 4, 13, 12, 30);
    asset.originalDate = new Date(2018, 3, 20, 17, 0);
    // act
    const bestDate = asset.bestDate();
    // assert
    expect(bestDate).toEqual(asset.originalDate);
  });
});
