//
// Copyright (c) 2025 Nathan Fiedler
//
import { Location } from './location.ts';

/**
 * Parameters for searching for assets based on various criteria.
 */
class SearchParams {
  tags: string[];
  locations: string[];
  mediaType: string | null;
  beforeDate: Date | null;
  afterDate: Date | null;
  sortField: SortField | null;
  sortOrder: SortOrder | null;

  constructor() {
    this.tags = [];
    this.locations = [];
    this.mediaType = null;
    this.beforeDate = null;
    this.afterDate = null;
    this.sortField = null;
    this.sortOrder = null;
  }

  addTag(tag: string): SearchParams {
    this.tags.push(tag);
    return this;
  }

  setTags(tags: string[]): SearchParams {
    this.tags = tags;
    return this;
  }

  addLocation(location: string): SearchParams {
    this.locations.push(location);
    return this;
  }

  setLocations(locations: string[]): SearchParams {
    this.locations = locations;
    return this;
  }

  setMediaType(mediaType: string): SearchParams {
    this.mediaType = mediaType;
    return this;
  }

  setBeforeDate(beforeDate: Date): SearchParams {
    this.beforeDate = beforeDate;
    return this;
  }

  setAfterDate(afterDate: Date): SearchParams {
    this.afterDate = afterDate;
    return this;
  }

  setSortField(sortField: SortField): SearchParams {
    this.sortField = sortField;
    return this;
  }

  setSortOrder(sortOrder: SortOrder): SearchParams {
    this.sortOrder = sortOrder;
    return this;
  }
}

/**
 * Field of the search results on which to sort.
 */
enum SortField {
  Date,
  Identifier,
  Filename,
  MediaType
}

/**
 * Order by which to sort the search results.
 */
enum SortOrder {
  Ascending,
  Descending
}

/**
 * Parameters for finding assets that are pending (no tags, caption, etc).
 */
class PendingParams {
  afterDate: Date | null = null;
  sortField: SortField | null = null;
  sortOrder: SortOrder | null = null;

  setAfterDate(afterDate: Date): PendingParams {
    this.afterDate = afterDate;
    return this;
  }

  setSortField(sortField: SortField): PendingParams {
    this.sortField = sortField;
    return this;
  }

  setSortOrder(sortOrder: SortOrder): PendingParams {
    this.sortOrder = sortOrder;
    return this;
  }
}

/**
 * Returned from the record repository when performing a query.
 */
class SearchResult {
  /** Asset identifier. */
  assetId: string;
  /** Original filename of the asset. */
  filename: string;
  /** Media type (formerly MIME type) of the asset. */
  mediaType: string;
  /** Location of the asset. */
  location: Location | null;
  /** Best date/time for the indexed asset. */
  datetime: Date;

  constructor(
    assetId: string,
    filename: string,
    mediaType: string,
    location: Location | null,
    datetime: Date
  ) {
    this.assetId = assetId;
    this.filename = filename;
    this.mediaType = mediaType;
    this.location = location;
    this.datetime = datetime;
  }
}

export { PendingParams, SearchParams, SortField, SortOrder, SearchResult };
