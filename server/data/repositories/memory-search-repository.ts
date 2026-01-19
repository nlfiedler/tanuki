//
// Copyright (c) 2026 Nathan Fiedler
//
import { SearchResult } from 'tanuki/server/domain/entities/search.ts';
import { type SearchRepository } from 'tanuki/server/domain/repositories/search-repository.ts';
import { LRUCache } from 'tanuki/server/shared/collections/lru-cache.ts';

/**
 * Repository for caching search and scan results using a weight LRU cache.
 */
class MemorySearchRepository implements SearchRepository {
  cache: LRUCache<string, SearchResult[]>;

  constructor() {
    // each search result will have a weight of 1; if a search result is roughly
    // 128 bytes in memory (that is a made up value but probably close), then a
    // megabyte of memory would hold 8,192 results; let's assume we can afford
    // to use 4 mb of memory for the cache
    this.cache = new LRUCache(32_768);
  }

  /** @inheritdoc */
  async put(key: string, results: SearchResult[]): Promise<void> {
    this.cache.set(key, results, results.length);
  }

  /** @inheritdoc */
  async get(key: string): Promise<SearchResult[] | undefined> {
    return this.cache.get(key);
  }

  /** @inheritdoc */
  async clear(): Promise<void> {
    this.cache.clear();
  }
}

export { MemorySearchRepository };
