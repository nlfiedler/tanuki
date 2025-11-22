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
import { SortOrder, SortField } from 'tanuki/server/domain/entities/SearchParams.ts';
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
export async function getOriginalDate(mimetype: string, filepath: string): Promise<number | null> {
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

/**
 * Sort the search results if the field is specified, in the order given.
 *
 * @param results search results to be sorted.
 * @param field field on which to sort the results.
 * @param order desired sort order.
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
