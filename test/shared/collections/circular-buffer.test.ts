//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, test } from 'bun:test';
import { CircularBuffer } from 'tanuki/server/shared/collections/circular-buffer.ts';

describe('CircularBuffer', function () {
  test('should overwrite oldest items once full', function () {
    // arrange
    const sut = new CircularBuffer<number>(8);
    // act
    for (let v = 0; v < 16; v++) {
      sut.enqueue(v);
    }
    // assert
    expect(sut.length).toEqual(8);
    expect(sut.isFull()).toBeTrue();
    expect(sut.peek()).toEqual(8);
    expect(sut.isFull()).toBeTrue();
    expect(sut.dequeue()).toEqual(8);
    expect(sut.isFull()).toBeFalse();
    expect(sut.isEmpty()).toBeFalse();
    expect(sut.dequeue()).toEqual(9);
    expect(sut.dequeue()).toEqual(10);
    expect(sut.dequeue()).toEqual(11);
    expect(sut.dequeue()).toEqual(12);
    expect(sut.dequeue()).toEqual(13);
    expect(sut.dequeue()).toEqual(14);
    expect(sut.dequeue()).toEqual(15);
    expect(sut.isEmpty()).toBeTrue();
    expect(sut.length).toEqual(0);
    expect(sut.peek()).toBeUndefined();
    expect(sut.dequeue()).toBeUndefined();
  });
});
