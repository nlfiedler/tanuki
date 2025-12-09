//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import * as helpers from './helpers.ts';
import { SearchParams } from 'tanuki/server/domain/entities/SearchParams.ts';
import { SearchResult } from 'tanuki/server/domain/entities/SearchResult.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/RecordRepository.ts';

export default ({
  recordRepository
}: {
  recordRepository: RecordRepository;
}) => {
  assert.ok(recordRepository, 'record repository must be defined');
  /**
   * Use case to perform queries against the database indices.
   *
   * If not specified, the sort order will be _ascending_ by default.
   *
   * @param params - search parameters.
   * @returns array of results containing selected fields from asset records.
   */
  return async (params: SearchParams): Promise<SearchResult[]> => {
    let results = await queryAssets(recordRepository, params);
    results = filterByDateRange(results, params);
    results = filterByLocations(results, params);
    results = filterByMediaType(results, params);
    helpers.sortSearchResults(results, params.sortField, params.sortOrder);
    return results;
  };
};

/** Perform an initial search of the assets. */
async function queryAssets(
  recordRepository: RecordRepository,
  params: SearchParams
): Promise<SearchResult[]> {
  // Perform the initial query using one of the criteria. Querying by tags is
  // the first choice since the search results do not contain the tags, so a
  // filter on tags is not possible.
  //
  // Any other field that is used to perform the initial search will be cleared
  // from the search parameters to prevent filtering by that same criteria,
  // which would be useless.
  if (params.tags.length > 0) {
    return recordRepository.queryByTags(params.tags);
  } else if (params.afterDate && params.beforeDate) {
    const after = params.afterDate;
    const before = params.beforeDate;
    params.afterDate = null;
    params.beforeDate = null;
    return recordRepository.queryDateRange(after, before);
  } else if (params.beforeDate) {
    const before = params.beforeDate;
    params.beforeDate = null;
    return recordRepository.queryBeforeDate(before);
  } else if (params.afterDate) {
    const after = params.afterDate;
    params.afterDate = null;
    return recordRepository.queryAfterDate(after);
  } else if (params.locations.length > 0) {
    const locations = params.locations;
    params.locations = [];
    return recordRepository.queryByLocations(locations);
  } else if (params.mediaType) {
    const mediaType = params.mediaType;
    params.mediaType = null;
    return recordRepository.queryByMediaType(mediaType);
  } else {
    return Promise.resolve([]);
  }
}

/** Filter the search results by date range, if specified by the parameters. */
function filterByDateRange(
  results: SearchResult[],
  params: SearchParams
): SearchResult[] {
  if (params.afterDate && params.beforeDate) {
    const a = params.afterDate;
    const b = params.beforeDate;
    return results.filter((r) => r.datetime > a && r.datetime < b);
  } else if (params.beforeDate) {
    const b = params.beforeDate;
    return results.filter((r) => r.datetime < b);
  } else if (params.afterDate) {
    const a = params.afterDate;
    return results.filter((r) => r.datetime > a);
  }
  return results;
}

/**
 * Filter the search results by location(s), if specified.
 *
 * Matches a result if it contains all of the specified location values. This
 * means that a search for "paris, texas" will turn up results that have both
 * "paris" and "texas" as part of the location entry, and not simply return
 * results that contain either "paris" or "texas".
 */
function filterByLocations(
  results: SearchResult[],
  params: SearchParams
): SearchResult[] {
  if (params.locations.length > 0) {
    // All filtering comparisons are case-insensitive for now, so both the input
    // and the index values are lowercased.
    const locations = params.locations.map((v) => v.toLowerCase());
    return results.filter((r) => {
      if (r.location) {
        // every given location search term must match some part of the asset
        // location (such as ["paris", "france"] versus ["paris", "texas"])
        return locations.every((l) => r.location?.partialMatch(l));
      }
      return false;
    });
  }
  return results;
}

/** Filter the search results by media type, if specified. */
function filterByMediaType(
  results: SearchResult[],
  params: SearchParams
): SearchResult[] {
  if (params.mediaType) {
    // All filtering comparisons are case-insensitive for now, so both the
    // input and the index values are lowercased.
    const mt = params.mediaType.toLowerCase();
    return results.filter((r) => r.mediaType.toLowerCase() == mt);
  }
  return results;
}
