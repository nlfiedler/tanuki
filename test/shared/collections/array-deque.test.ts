//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, test } from 'bun:test';
import { ArrayDeque } from 'tanuki/server/shared/collections/array-deque.ts';

describe('ArrayDeque', function () {
  test('should grow the array once full', function () {
    // arrange
    const sut = new ArrayDeque(8);
    // act
    for (let v = 0; v < 16; v++) {
      sut.enqueue(v);
    }
    // assert
    expect(sut.length).toEqual(16);
    expect(sut.peek()).toEqual(0);
    for (let v = 0; v < 16; v++) {
      expect(sut.dequeue()).toEqual(v);
    }
    expect(sut.length).toEqual(0);
    expect(sut.peek()).toBeUndefined();
    expect(sut.dequeue()).toBeUndefined();
  });

  test('should wrap around and grow as needed', function () {
    // arrange
    const sut = new ArrayDeque(8);
    // act
    for (let v = 0; v < 16; v++) {
      sut.enqueue(v);
    }
    for (let v = 0; v < 8; v++) {
      expect(sut.dequeue()).toEqual(v);
    }
    for (let v = 16; v < 32; v++) {
      sut.enqueue(v);
    }
    // assert
    expect(sut.length).toEqual(24);
    expect(sut.peek()).toEqual(8);
    for (let v = 8; v < 32; v++) {
      expect(sut.dequeue()).toEqual(v);
    }
    expect(sut.length).toEqual(0);
    expect(sut.peek()).toBeUndefined();
    expect(sut.dequeue()).toBeUndefined();
  });
});
