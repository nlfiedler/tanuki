//
// Copyright (c) 2025 Nathan Fiedler
//
import crypto from 'node:crypto';
import fs from 'node:fs/promises';
import path from 'node:path';
import dayjs from 'dayjs';
import ExifReader from 'exifreader';
import mime from 'mime';
import { ulid } from 'ulid';
import {
  Coordinates,
  Geocoded,
  Location
} from 'tanuki/server/domain/entities/Location.ts';
import {
  SortOrder,
  SortField
} from 'tanuki/server/domain/entities/SearchParams.ts';
import { SearchResult } from 'tanuki/server/domain/entities/SearchResult.ts';

/**
 * Compute the hash digest of the file at the given path. The value will have
 * the algorithm prefixed with a hyphen separator (e.g. sha256-xxx).
 *
 * @param filepath - path of file for which to generate hash digest.
 * @return hash digest of file.
 */
export async function checksumFile(filepath: string): Promise<string> {
  const algo = 'sha256';
  const hash = crypto.createHash(algo);
  const handle = await fs.open(filepath);
  const stream = handle.createReadStream();
  for await (const chunk of stream) {
    hash.update(chunk);
  }
  return `${algo}-${hash.digest('hex')}`;
}

/**
 * Use the datetime and media type to produce a relative path, and return as a
 * base64 encoded value, suitable as an identifier.
 *
 * The decoded identifier is suitable to be used as a file path within blob
 * storage. The generated filename will be universally unique and the path will
 * ensure assets are distributed across directories to avoid congestion.
 *
 * This is _not_ a pure function, since it involves a random number. It does,
 * however, avoid any possibility of name collisions.
 *
 * @param datetime - date/time of import.
 * @param mimetype - media type of the asset.
 * @return newly generated asset identifier.
 */
export function newAssetId(datetime: Date, mimetype: string): string {
  // Round the date/time down to the nearest quarter hour (e.g. 21:50 becomes
  // 21:45, 08:10 becomes 08:00) to avoid creating many directories with only
  // a few assets in them.
  const roundate = new Date(
    datetime.getFullYear(),
    datetime.getMonth(),
    datetime.getDate(),
    datetime.getHours(),
    Math.floor(datetime.getMinutes() / 15) * 15
  );
  const datepath = dayjs(roundate).format('YYYY/MM/DD/HHmm');
  // For our purposes, any extension that may have been attached to the incoming
  // filename is irrelevant when we are given the media type, from which we can
  // compute the most appropriate extension.
  const name = ulid() + '.' + mime.getExtension(mimetype);
  const assetpath = path.join(datepath, name).toLowerCase();
  const buf = Buffer.from(assetpath, 'utf8');
  return buf.toString('base64');
}

/**
 * Return the guessed media type based on the extension.
 *
 * @param extension - filename extension for guessing media type.
 * @returns guessed media type.
 */
export function inferMediaType(extension: string): string {
  const guess = mime.getType(extension);
  if (guess === null) {
    if (extension === 'aae') {
      return 'text/xml';
    }
    return 'application/octet-stream';
  }
  return guess;
}

/**
 * Extract the original date/time from the asset. For images that contain Exif
 * data, returns the parsed DateTimeOriginal value. For supported video files,
 * returns the creation_time value.
 *
 * @param mimetype - the mime type of the file.
 * @param filepath - path to file to be examined.
 * @returns Milliseconds since epoch if successful, null otherwise.
 */
export async function getOriginalDate(
  mimetype: string,
  filepath: string
): Promise<number | null> {
  try {
    if (mimetype.startsWith('image/')) {
      const tags = await ExifReader.load(filepath);
      if ('DateTimeOriginal' in tags) {
        return parseExifDate(tags.DateTimeOriginal!.description);
      }
      // } else if (mimetype.startsWith('video/')) {
      //   return getCreationTime(filepath);
    }
  } catch (err) {
    // failed to read file/data for whatever reason
    return null;
  }
  return null;
}

const DATE_REGEXP = new RegExp(
  // https://www.media.mit.edu/pia/Research/deepview/exif.html -- DateTime
  //  yyyy  :    MM  :    dd       HH  :    mm  :    ss
  '^(\\d{4}):(\\d{2}):(\\d{2}) (\\d{2}):(\\d{2}):(\\d{2})'
);

/**
 * Convert the Exif formatted date/time into UTC milliseconds, or null if the
 * value could not be parsed successfully.
 */
function parseExifDate(value: string): number | null {
  const m = DATE_REGEXP.exec(value);
  if (m == null) {
    return null;
  }
  return Date.UTC(
    parseInt(m[1] || '0'),
    parseInt(m[2] || '0') - 1, // Date uses zero-based month
    parseInt(m[3] || '0'),
    parseInt(m[4] || '0'),
    parseInt(m[5] || '0')
  );
}

// help TypeScript check the type of the latitude/longitude data
type ExifNumberPairs = [[number, number], [number, number], [number, number]];

/**
 * Extract the GPS coordinates from the asset, if any.
 *
 * @param mimetype - the mime type of the file.
 * @param filepath - path to file to be examined.
 * @returns GPS coordinates if any, null otherwise.
 */
export async function getCoordinates(
  mimetype: string,
  filepath: string
): Promise<Coordinates | null> {
  try {
    if (mimetype.startsWith('image/')) {
      const tags = await ExifReader.load(filepath);
      if (Array.isArray(tags.GPSLatitudeRef?.value)) {
        const coords = new Coordinates('N', [0, 0, 0], 'E', [0, 0, 0]);
        if (tags.GPSLatitudeRef.value[0] === 'S') {
          coords.setLatitudeRef('S');
        }
        if (Array.isArray(tags.GPSLatitude?.value)) {
          const numbers = tags.GPSLatitude?.value as ExifNumberPairs;
          coords.setLatitudeDegrees(numbers[0][0] / numbers[0][1]);
          coords.setLatitudeMinutes(numbers[1][0] / numbers[1][1]);
          coords.setLatitudeSeconds(numbers[2][0] / numbers[2][1]);
        }
        if (Array.isArray(tags.GPSLongitudeRef?.value)) {
          if (tags.GPSLongitudeRef.value[0] === 'W') {
            coords.setLongitudeRef('W');
          }
          if (Array.isArray(tags.GPSLongitude?.value)) {
            const numbers = tags.GPSLongitude?.value as ExifNumberPairs;
            coords.setLongitudeDegrees(numbers[0][0] / numbers[0][1]);
            coords.setLongitudeMinutes(numbers[1][0] / numbers[1][1]);
            coords.setLongitudeSeconds(numbers[2][0] / numbers[2][1]);
          }
        }
        return coords;
      }
    }
  } catch (err) {
    // failed to read file/data for whatever reason
    return null;
  }
  return null;
}

//
// Example of EXIF coordinates from image file:
//
//  GPSLatitudeRef: {
//     id: 1,
//     value: [ "N" ],
//     description: "North latitude",
//   },
//   GPSLatitude: {
//     id: 2,
//     value: [
//       [ 37, 1 ], [ 42, 1 ], [ 3193, 100 ]
//     ],
//     description: 37.708869444444446,
//   },
//   GPSLongitudeRef: {
//     id: 3,
//     value: [ "W" ],
//     description: "West longitude",
//   },
//   GPSLongitude: {
//     id: 4,
//     value: [
//       [ 122, 1 ], [ 3, 1 ], [ 4772, 100 ]
//     ],
//     description: 122.06325555555556,
//   },

/**
 * Sort the search results if the field is specified, in the order given.
 *
 * @param results search results to be sorted.
 * @param field field on which to sort the results.
 * @param order desired sort order (ascending by default).
 */
export function sortSearchResults(
  results: SearchResult[],
  field: SortField | null,
  order: SortOrder | null
) {
  if (order === null) {
    order = SortOrder.Ascending;
  }
  if (field === SortField.Date) {
    if (order === SortOrder.Ascending) {
      results.sort((a, b) => a.datetime.getTime() - b.datetime.getTime());
    } else {
      results.sort((a, b) => b.datetime.getTime() - a.datetime.getTime());
    }
  } else if (field === SortField.Identifier) {
    if (order === SortOrder.Ascending) {
      results.sort((a, b) => a.assetId.localeCompare(b.assetId));
    } else {
      results.sort((a, b) => b.assetId.localeCompare(a.assetId));
    }
  } else if (field === SortField.Filename) {
    if (order === SortOrder.Ascending) {
      results.sort((a, b) => a.filename.localeCompare(b.filename));
    } else {
      results.sort((a, b) => b.filename.localeCompare(a.filename));
    }
  } else if (field === SortField.MediaType) {
    if (order === SortOrder.Ascending) {
      results.sort((a, b) => a.mediaType.localeCompare(b.mediaType));
    } else {
      results.sort((a, b) => b.mediaType.localeCompare(a.mediaType));
    }
  }
}

/**
 * Returns the merged location object. Blank input fields will clear the asset
 * fields. If the input is null, the asset value is returned, and vice versa.
 *
 * @param asset - location from the asset entity.
 * @param input - location with new values.
 * @returns location entity with merged fields.
 */
export function mergeLocations(
  asset?: Readonly<Location> | null,
  input?: Location | null
): Location | null | undefined {
  if (asset) {
    if (input) {
      const outgoing = Location.fromRaw(asset.label, asset.city, asset.region);
      // set or clear the label field
      if (input.label !== null) {
        if (input.label.length == 0) {
          outgoing.label = null;
        } else {
          outgoing.label = input.label;
        }
      }
      // set or clear the city field
      if (input.city !== null) {
        if (input.city.length == 0) {
          outgoing.city = null;
        } else {
          outgoing.city = input.city;
        }
      }
      // set or clear the region field
      if (input.region !== null) {
        if (input.region.length == 0) {
          outgoing.region = null;
        } else {
          outgoing.region = input.region;
        }
      }
      return outgoing;
    } else {
      // input was null, return original value
      return asset;
    }
  }
  // original value is undefined, return input as-is
  return input;
}

/**
 * Convert the geocoded location to the domain version.
 *
 * @param geocoded - result from reverse geocoding, city must not be null.
 * @returns Location entity with appropriate city and region.
 */
export function locationFromGeocoded(geocoded: Geocoded): Location {
  const loc = Location.fromRaw(null, geocoded.city!, null);
  if (geocoded.city && geocoded.region) {
    const city = geocoded.city.toLocaleLowerCase();
    const region = geocoded.region.toLocaleLowerCase();
    // replace region with country if it is largely redundant
    if (city == region || region.startsWith(city) || region.endsWith(city)) {
      loc.region = geocoded.country;
    } else {
      loc.region = geocoded.region;
    }
  } else if (geocoded.region) {
    // no city but has region and possibly country, promote the values since the
    // domain entity does not have a country
    loc.city = geocoded.region;
    loc.region = geocoded.country;
  }
  return loc;
}

export type CaptionParts = {
  tags: string[];
  location: Location | null;
};

/**
 * Parse the given caption text into a list of tags and a location.
 *
 * @returns object with tags and location values.
 */
export function parseCaption(caption: string): CaptionParts {
  const lexer = new CaptionLexer(caption);
  let fun: LexerFun = lexStart;
  while (fun) {
    fun = fun(lexer);
  }
  const location = lexer.location ? Location.parse(lexer.location) : null;
  return { tags: lexer.tags, location };
}

class CaptionLexer {
  input: string;
  length: number;
  cursor: number;
  width: number;
  tags: string[];
  location: string | undefined;

  constructor(input: string) {
    this.input = input;
    this.length = input.length;
    this.cursor = 0;
    this.width = 0;
    this.tags = [];
  }

  next(): string | undefined {
    if (this.cursor >= this.length) {
      // so backup() does nothing at the end of the input
      this.width = 0;
      return undefined;
    }
    const ch = this.input[this.cursor];
    this.cursor++;
    this.width = 1;
    return ch;
  }

  backup() {
    if (this.cursor > 0) {
      this.cursor -= this.width;
    }
  }

  peek(): string | undefined {
    return this.input[this.cursor];
  }
}

// Defining a recursive type definition in TypeScript seems to work if the
// recursive reference is after the other possible variants.
type LexerFun = null | ((l: CaptionLexer) => LexerFun);

function lexStart(l: CaptionLexer): LexerFun {
  let ch = l.next();
  while (ch) {
    if (ch == '#') {
      return lexTag;
    } else if (ch == '@') {
      return lexLocation;
    }
    ch = l.next();
  }
  return null;
}

function lexTag(l: CaptionLexer): LexerFun {
  const tag = acceptIdentifier(l);
  l.tags.push(tag);
  return lexStart;
}

function lexLocation(l: CaptionLexer): LexerFun {
  let ch = l.peek();
  if (ch) {
    if (ch == '"') {
      // ignore the opening quote
      l.next();
      // scan until the next quote is found
      let ident = '';
      ch = l.peek();
      while (ch) {
        if (ch == '"') {
          break;
        } else {
          ident = ident.concat(ch);
          l.next();
        }
        ch = l.peek();
      }
      l.location = ident;
    } else {
      const location = acceptIdentifier(l);
      l.location = location;
    }
  }
  return lexStart;
}

/** Processes the text as a tag or location. */
function acceptIdentifier(l: CaptionLexer): string {
  let ident = '';
  let ch = l.peek();
  while (ch) {
    if (isDelimiter(ch)) {
      break;
    } else {
      ident = ident.concat(ch);
      l.next();
    }
    ch = l.peek();
  }
  return ident;
}

/** Returns true if `ch` is a delimiter character. */
function isDelimiter(ch: string): boolean {
  return (
    ch === ' ' ||
    ch === '.' ||
    ch === ',' ||
    ch === ';' ||
    ch === '(' ||
    ch === ')' ||
    ch === '"'
  );
}
