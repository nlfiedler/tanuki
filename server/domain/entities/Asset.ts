//
// Copyright (c) 2025 Nathan Fiedler
//
import { Location } from './Location.ts';

/**
 * Asset entity represents an image, video, or other file in the system.
 */
class Asset {
  /** The unique identifier of the asset. */
  key: string;
  /** Hash digest of the asset contents. */
  checksum: string;
  /** Original filename of the asset. */
  filename: string;
  /** Size of the asset in bytes. */
  byteLength: number;
  /** Media type (formerly MIME type) of the asset. */
  mediaType: string;
  /** Set of user-assigned labels for the asset. */
  tags: string[];
  /** Date when the asset was imported. */
  importDate: Date;
  /** Caption provided by the user. */
  caption: string | null;
  /** Location information for the asset. */
  location: Location | null;
  /** User-specified date of the asset. */
  userDate: Date | null;
  /** Date of the asset as extracted from metadata. */
  originalDate: Date | null;

  constructor(key: string) {
    this.key = key;
    this.checksum = "";
    this.filename = "";
    this.byteLength = 0;
    this.mediaType = "";
    this.tags = [];
    this.importDate = new Date();
    this.caption = null;
    this.location = null;
    this.userDate = null;
    this.originalDate = null;
  }

  /**
   * Returns the most accurate date for the asset, starting with the
   * user-defined date, then the date read from the asset itself, and lastly the
   * time of import.
   *
   * @returns The most accurate date available for this asset.
   */
  bestDate(): Date {
    if (this.userDate !== null) {
      return this.userDate;
    } else if (this.originalDate !== null) {
      return this.originalDate;
    }
    return this.importDate;
  }

  /**
   * Relative path of the asset within the asset store.
   * 
   * @returns relative path of the asset.
   */
  filepath(): string {
    const buf = Buffer.from(this.key, 'base64');
    return buf.toString('utf8');
  }

  setChecksum(checksum: string): Asset {
    this.checksum = checksum;
    return this;
  }

  setFilename(filename: string): Asset {
    this.filename = filename;
    return this;
  }

  setByteLength(byteLength: number): Asset {
    this.byteLength = byteLength;
    return this;
  }

  setMediaType(mediaType: string): Asset {
    this.mediaType = mediaType;
    return this;
  }

  setTags(tags: string[]): Asset {
    this.tags = tags;
    return this;
  }

  setImportDate(importDate: Date): Asset {
    this.importDate = importDate;
    return this;
  }

  setCaption(caption: string): Asset {
    this.caption = caption;
    return this;
  }

  setLocation(location: Location): Asset {
    this.location = location;
    return this;
  }

  setUserDate(userDate: Date): Asset {
    this.userDate = userDate;
    return this;
  }

  setOriginalDate(originalDate: Date): Asset {
    this.originalDate = originalDate;
    return this;
  }
}

export { Asset };
