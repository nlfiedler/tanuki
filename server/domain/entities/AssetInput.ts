//
// Copyright (c) 2025 Nathan Fiedler
//
import { Location } from './Location.ts';

/**
 * Asset entity represents an image, video, or other file in the system.
 */
class AssetInput {
  /** Identifier for the asset to be updated. */
  key: string;
  /**
   * Any values here will replace the existing values, and are sorted and
   * de-duplicated.
   */
  tags: string[] | null;
  /**
   * Any value here overwrites the caption in the asset. If the caption
   * contains any #tags they will be merged with the tags in the asset (or in
   * the input, if given). If the caption contains an @location or @"location"
   * then it will replace the asset location, if it has not been set. That is,
   * the caption only enhances, never clobbers.
   */
  caption: string | null;
  /**
   * Any value here overwrites the location in the asset. This field takes
   * precedence over any @location value in the caption.
   */
  location: Location | null;
  /** Any value here overwrites the user-defined date. */
  datetime: Date | null;
  /** Any value here overwrites the media_type property. */
  mediaType: string | null;
  /** Any value here overwrites the filename property. */
  filename: string | null;

  constructor(key: string) {
    this.key = key;
    this.tags = null;
    this.caption = null;
    this.location = null;
    this.datetime = null;
    this.mediaType = '';
    this.filename = '';
  }

  /** Return `true` if any of the fields have a value. */
  hasValues(): boolean {
    return (
      (this.tags !== null && this.tags.length > 0) ||
      this.caption !== null ||
      this.location !== null ||
      this.datetime !== null ||
      this.mediaType !== null ||
      this.filename !== null
    );
  }

  setFilename(filename: string): AssetInput {
    this.filename = filename;
    return this;
  }

  setMediaType(mediaType: string): AssetInput {
    this.mediaType = mediaType;
    return this;
  }

  addTag(tag: string): AssetInput {
    if (this.tags === null) {
      this.tags = [tag];
    } else {
      this.tags.push(tag);
    }
    return this;
  }

  setTags(tags: string[]): AssetInput {
    this.tags = tags;
    return this;
  }

  setCaption(caption: string): AssetInput {
    this.caption = caption;
    return this;
  }

  setLocation(location: Location): AssetInput {
    this.location = location;
    return this;
  }

  setDatetime(datetime: Date): AssetInput {
    this.datetime = datetime;
    return this;
  }
}

export { AssetInput };
