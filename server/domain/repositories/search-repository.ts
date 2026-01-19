//
// Copyright (c) 2026 Nathan Fiedler
//
import { SearchResult } from 'tanuki/server/domain/entities/search.ts';

/**
 * Repository for caching search or scan results using a weight LRU cache.
 */
interface SearchRepository {
  /** Cache the given search results for easy retrieval later. */
  put(key: string, results: SearchResult[]): Promise<void>;

  /** Retrieve the cached search results, if available. */
  get(key: string): Promise<SearchResult[] | undefined>;

  /** Clear all cached search results. */
  clear(): Promise<void>;
}

export { type SearchRepository };
