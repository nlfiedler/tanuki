//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, test } from 'bun:test';
import { LRUCache } from 'tanuki/server/shared/collections/lru-cache.ts';

describe('LRUCache', function () {
  test('should evict oldest item', function () {
    const cache = new LRUCache<number, string>(10);
    expect(cache.length).toEqual(0);
    expect(cache.get(1)).toBeUndefined();

    // start with the basics, add the items up to the maximum weight
    cache.set(1, 'one', 3);
    cache.set(2, 'two', 4);
    cache.set(3, 'three', 2);
    expect(cache.length).toEqual(3);
    expect(cache.currentWeight).toEqual(9);

    // add a new item, now '1' should be evicted
    cache.set(4, 'four', 3);
    expect(cache.length).toEqual(3);
    expect(cache.currentWeight).toEqual(9);
    expect(cache.has(1)).toBeFalse();
    expect(cache.get(1)).toBeUndefined();

    // now "get" an item to move it to the front
    expect(cache.get(2)).toEqual('two');

    // add a new item, now '3' will be evicted
    cache.set(5, 'five', 2);
    expect(cache.length).toEqual(3);
    expect(cache.currentWeight).toEqual(9);
    expect(cache.has(3)).toBeFalse();
    expect(cache.get(3)).toBeUndefined();
    expect(cache.has(2)).toBeTrue();
    expect(cache.has(4)).toBeTrue();
    expect(cache.has(5)).toBeTrue();
  });

  test('should evict oldest item, all weights are 1', function () {
    const cache = new LRUCache<number, string>(3);
    expect(cache.length).toEqual(0);
    expect(cache.get(1)).toBeUndefined();

    // start with the basics, add the items up to the maximum weight
    cache.set(1, 'one');
    cache.set(2, 'two');
    cache.set(3, 'three');
    expect(cache.length).toEqual(3);
    expect(cache.currentWeight).toEqual(3);

    // add a new item, now '1' should be evicted
    cache.set(4, 'four');
    expect(cache.length).toEqual(3);
    expect(cache.currentWeight).toEqual(3);
    expect(cache.has(1)).toBeFalse();
    expect(cache.get(1)).toBeUndefined();

    // now "get" an item to move it to the front
    expect(cache.get(2)).toEqual('two');

    // add a new item, now '3' will be evicted
    cache.set(5, 'five');
    expect(cache.length).toEqual(3);
    expect(cache.currentWeight).toEqual(3);
    expect(cache.has(3)).toBeFalse();
    expect(cache.get(3)).toBeUndefined();
    expect(cache.has(2)).toBeTrue();
    expect(cache.has(4)).toBeTrue();
    expect(cache.has(5)).toBeTrue();
  });

  test('should track weight accurately', function () {
    const cache = new LRUCache<number, string>(10);
    // replacing a value for the same key should sub/add weights accordingly
    cache.set(1, 'one', 3);
    cache.set(1, 'eins', 4);
    cache.set(1, 'ichi', 4);
    cache.set(1, 'ii', 2);
    cache.set(1, 'uno', 3);
    expect(cache.length).toEqual(1);
    expect(cache.currentWeight).toEqual(3);
    expect(cache.has(1)).toBeTrue();
    expect(cache.get(1)).toEqual('uno');
  });

  test('should evict all items if necessary', function () {
    // the cache will only hold items that fit, even if it's just one item
    const cache = new LRUCache<number, string>(10);
    cache.set(1, 'one', 11);
    expect(cache.length).toEqual(0);
    expect(cache.currentWeight).toEqual(0);
    expect(cache.has(1)).toBeFalse();
    expect(cache.get(1)).toBeUndefined();
  });

  test('should clear all items', function () {
    const cache = new LRUCache<number, string>(10);
    cache.set(1, 'one', 3);
    cache.set(2, 'two', 4);
    cache.set(3, 'three', 2);
    expect(cache.length).toEqual(3);
    expect(cache.currentWeight).toEqual(9);
    cache.clear();
    expect(cache.length).toEqual(0);
    expect(cache.currentWeight).toEqual(0);
    expect(cache.has(1)).toBeFalse();
    expect(cache.has(2)).toBeFalse();
    expect(cache.has(3)).toBeFalse();
  });
});
