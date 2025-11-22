//
// Copyright (c) 2025 Nathan Fiedler
//
import { Location } from './Location.ts';

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

  constructor(assetId: string, filename: string, mediaType: string, location: Location | null, datetime: Date) {
    this.assetId = assetId;
    this.filename = filename;
    this.mediaType = mediaType;
    this.location = location;
    this.datetime = datetime;
  }
}

export { SearchResult };
