//
// Copyright (c) 2025 Nathan Fiedler
//
import crypto from 'node:crypto';
import { createReadStream } from 'node:fs';
import fs from 'node:fs/promises';
import path from 'node:path';
import dayjs from 'dayjs';
import ExifReader from 'exifreader';
import mime from 'mime';
import { createFile, Log, MP4BoxBuffer, type Movie } from 'mp4box';
import { ulid } from 'ulid';
import { AssetMetadata } from 'tanuki/server/domain/entities/asset-metadata.ts';
import {
  Coordinates,
  Geocoded,
  Location
} from 'tanuki/server/domain/entities/location.ts';
import { SortOrder, SortField } from 'tanuki/server/domain/entities/search.ts';
import { SearchResult } from 'tanuki/server/domain/entities/search.ts';

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
 * base64url encoded value, suitable as an identifier.
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
  return buf.toString('base64url');
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
    } else if (mimetype.startsWith('video/')) {
      return getCreationTime(filepath);
    }
  } catch {
    // failed to read file/data for whatever reason
    return null;
  }
  return null;
}

// Silence mp4box's BoxParser console output. Its setLogLevel() bottoms out at
// ERROR, so warnings like "Invalid box type" still print; we only care about
// the resolved info or a null result.
Log.error = () => {};
Log.warn = () => {};
Log.info = () => {};
Log.debug = () => {};

/**
 * Read the creation_time from the movie header of an MP4 or QuickTime (MOV)
 * file. Uses mp4box.js to parse the file incrementally; returns null for files
 * that are not valid MP4/QuickTime or that lack a parseable mvhd box.
 *
 * @param filepath - path to the video file.
 * @returns Milliseconds since epoch if successful, null otherwise.
 */
function getCreationTime(filepath: string): Promise<number | null> {
  return new Promise((resolve) => {
    const mp4boxfile = createFile();
    let settled = false;
    const finish = (value: number | null) => {
      if (!settled) {
        settled = true;
        resolve(value);
      }
    };
    mp4boxfile.onReady = (info: Movie) => {
      finish(info.created instanceof Date ? info.created.getTime() : null);
    };
    mp4boxfile.onError = () => finish(null);

    const stream = createReadStream(filepath, { highWaterMark: 65_536 });
    let fileOffset = 0;
    stream.on('data', (chunk: Buffer) => {
      if (settled) return;
      const ab = MP4BoxBuffer.fromArrayBuffer(
        chunk.buffer.slice(
          chunk.byteOffset,
          chunk.byteOffset + chunk.byteLength
        ),
        fileOffset
      );
      fileOffset += chunk.length;
      try {
        mp4boxfile.appendBuffer(ab);
      } catch {
        finish(null);
      }
    });
    stream.on('end', () => {
      try {
        mp4boxfile.flush();
      } catch {
        // ignore flush errors; finish(null) handles it
      }
      finish(null);
    });
    stream.on('error', () => finish(null));
  });
}

/**
 * Variant of {@link getCreationTime} that reads from a caller-supplied byte
 * range fetcher rather than a local file path. Used to extract the creation
 * time from a video blob that may live on a remote store, without having to
 * fetch the entire file.
 *
 * @param fetcher - callback that returns a buffer of bytes for the inclusive
 *   `[start, end]` range; an empty buffer indicates EOF.
 * @returns Milliseconds since epoch if successful, null otherwise.
 */
export async function getCreationTimeFromBlob(
  fetcher: (start: number, end: number) => Promise<Buffer>
): Promise<number | null> {
  const mp4boxfile = createFile();
  let result: number | null | undefined;
  mp4boxfile.onReady = (info: Movie) => {
    result = info.created instanceof Date ? info.created.getTime() : null;
  };
  mp4boxfile.onError = () => {
    if (result === undefined) result = null;
  };
  const CHUNK_SIZE = 65_536;
  let offset = 0;
  while (result === undefined) {
    let chunk: Buffer;
    try {
      chunk = await fetcher(offset, offset + CHUNK_SIZE - 1);
    } catch {
      return null;
    }
    if (chunk.length === 0) {
      try {
        mp4boxfile.flush();
      } catch {
        // ignore flush errors; falls through to null below
      }
      return result ?? null;
    }
    const ab = MP4BoxBuffer.fromArrayBuffer(
      chunk.buffer.slice(chunk.byteOffset, chunk.byteOffset + chunk.byteLength),
      offset
    );
    offset += chunk.length;
    try {
      mp4boxfile.appendBuffer(ab);
    } catch {
      return null;
    }
  }
  return result;
}

const DATE_REGEXP = new RegExp(
  // https://www.media.mit.edu/pia/Research/deepview/exif.html -- DateTime
  //  yyyy  :    MM  :    dd       HH  :    mm  :    ss
  String.raw`^(\d{4}):(\d{2}):(\d{2}) (\d{2}):(\d{2}):(\d{2})`
);

// EXIF OffsetTimeOriginal values like "+09:00" or "-08:00", or "Z".
const OFFSET_REGEXP = /^([+-])(\d{2}):(\d{2})$/;

/**
 * Convert the Exif formatted date/time into UTC milliseconds, or null if the
 * value could not be parsed successfully. If `offset` is supplied (a string
 * like "+09:00", "-08:00", or "Z"), the EXIF date is treated as local time in
 * that offset and converted to UTC. If `offset` is omitted, the date is
 * treated as already in UTC — historically the only behavior, kept as a
 * fallback for files without `OffsetTimeOriginal`.
 */
function parseExifDate(value: string, offset?: string | null): number | null {
  const m = DATE_REGEXP.exec(value);
  if (m == null) {
    return null;
  }
  const utcGuess = Date.UTC(
    Number.parseInt(m[1] || '0'),
    Number.parseInt(m[2] || '0') - 1, // Date uses zero-based month
    Number.parseInt(m[3] || '0'),
    Number.parseInt(m[4] || '0'),
    Number.parseInt(m[5] || '0'),
    Number.parseInt(m[6] || '0')
  );
  if (!offset) return utcGuess;
  if (offset === 'Z') return utcGuess;
  const om = OFFSET_REGEXP.exec(offset);
  if (om == null) return utcGuess;
  const sign = om[1] === '-' ? -1 : 1;
  const offsetMinutes =
    sign *
    (Number.parseInt(om[2] || '0') * 60 + Number.parseInt(om[3] || '0'));
  // `utcGuess` reads the local-clock components as if they were UTC. To get
  // true UTC, subtract the offset (a +09:00 local time is 9 hours ahead of
  // UTC, so true UTC = local - 9h).
  return utcGuess - offsetMinutes * 60 * 1000;
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
  } catch {
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
 * Combined output of reading the EXIF tags from an image file.
 */
export type ImageInfo = {
  originalDate: number | null;
  coordinates: Coordinates | null;
  metadata: AssetMetadata;
};

/**
 * Single-pass EXIF read for an image file. Returns the original date (as
 * milliseconds since epoch, offset-aware when `OffsetTimeOriginal` is
 * present), GPS coordinates, and an `AssetMetadata` populated from the file's
 * tags. Returns null if EXIF could not be read at all.
 *
 * Strips embedded thumbnails / preview blobs from the raw tag map before
 * storing it on the metadata, so the persisted `raw` field stays compact.
 */
export async function extractImageInfo(
  filepath: string
): Promise<ImageInfo | null> {
  try {
    const tags: any = await ExifReader.load(filepath);
    return parseImageTags(tags);
  } catch {
    return null;
  }
}

function tagDescription(tag: any): string | null {
  if (!tag) return null;
  if (typeof tag.description === 'string' && tag.description.length > 0) {
    return tag.description.trim();
  }
  if (Array.isArray(tag.value) && tag.value.length > 0) {
    return String(tag.value[0]).trim();
  }
  if (typeof tag.value === 'string') return tag.value.trim();
  if (typeof tag.value === 'number') return String(tag.value);
  return null;
}

function tagNumber(tag: any): number | null {
  if (!tag) return null;
  if (typeof tag.value === 'number') return tag.value;
  if (Array.isArray(tag.value) && tag.value.length > 0) {
    const v = tag.value[0];
    if (typeof v === 'number') return v;
    // rational [num, den]
    if (Array.isArray(v) && v.length === 2 && typeof v[0] === 'number') {
      const den = typeof v[1] === 'number' && v[1] !== 0 ? v[1] : 1;
      return v[0] / den;
    }
  }
  if (typeof tag.description === 'string') {
    const n = Number.parseFloat(tag.description);
    if (!Number.isNaN(n)) return n;
  }
  return null;
}

export function parseImageTags(tags: any): ImageInfo {
  const metadata = new AssetMetadata();

  metadata.cameraMake = tagDescription(tags.Make);
  metadata.cameraModel = tagDescription(tags.Model);
  metadata.lensMake = tagDescription(tags.LensMake);
  metadata.lensModel = tagDescription(tags.LensModel);
  metadata.exposureTime = tagDescription(tags.ExposureTime);
  metadata.fNumber = tagNumber(tags.FNumber);
  metadata.iso = tagNumber(tags.ISOSpeedRatings);
  metadata.focalLength35mm =
    tagNumber(tags.FocalLengthIn35mmFilm) ?? tagNumber(tags.FocalLength);
  metadata.originalDateOffset = tagDescription(tags.OffsetTimeOriginal);

  // Image dimensions adjusted for EXIF Orientation.
  const orientation = tagNumber(tags.Orientation) ?? 1;
  // ExifReader exposes dimensions under several keys depending on the source
  // segment: PixelXDimension/PixelYDimension (EXIF), ImageWidth/ImageLength
  // (TIFF), and 'Image Width'/'Image Height' (file-level / JFIF).
  const rawW =
    tagNumber(tags.PixelXDimension) ??
    tagNumber(tags.ImageWidth) ??
    tagNumber(tags['Image Width']);
  const rawH =
    tagNumber(tags.PixelYDimension) ??
    tagNumber(tags.ImageLength) ??
    tagNumber(tags.ImageHeight) ??
    tagNumber(tags['Image Height']);
  if (rawW !== null && rawH !== null) {
    const sideways = orientation >= 5 && orientation <= 8;
    metadata.displayWidth = sideways ? rawH : rawW;
    metadata.displayHeight = sideways ? rawW : rawH;
  }

  // Coordinates (also returned separately so the import path can geocode).
  const coordinates = parseGpsCoords(tags);
  if (coordinates) {
    const [lat, lon] = coordinates.intoDecimals();
    metadata.gpsLatitude = lat;
    metadata.gpsLongitude = lon;
  }

  // Original date, offset-aware when OffsetTimeOriginal is present.
  let originalDate: number | null = null;
  if (tags.DateTimeOriginal) {
    originalDate = parseExifDate(
      tags.DateTimeOriginal.description,
      metadata.originalDateOffset
    );
  }

  // Strip large embedded blobs from the raw tag map before storing.
  metadata.raw = stripRawImageTags(tags);

  return { originalDate, coordinates, metadata };
}

function parseGpsCoords(tags: any): Coordinates | null {
  if (!Array.isArray(tags.GPSLatitudeRef?.value)) return null;
  const coords = new Coordinates('N', [0, 0, 0], 'E', [0, 0, 0]);
  if (tags.GPSLatitudeRef.value[0] === 'S') coords.setLatitudeRef('S');
  if (Array.isArray(tags.GPSLatitude?.value)) {
    const numbers = tags.GPSLatitude.value as ExifNumberPairs;
    coords.setLatitudeDegrees(numbers[0][0] / numbers[0][1]);
    coords.setLatitudeMinutes(numbers[1][0] / numbers[1][1]);
    coords.setLatitudeSeconds(numbers[2][0] / numbers[2][1]);
  }
  if (Array.isArray(tags.GPSLongitudeRef?.value)) {
    if (tags.GPSLongitudeRef.value[0] === 'W') coords.setLongitudeRef('W');
    if (Array.isArray(tags.GPSLongitude?.value)) {
      const numbers = tags.GPSLongitude.value as ExifNumberPairs;
      coords.setLongitudeDegrees(numbers[0][0] / numbers[0][1]);
      coords.setLongitudeMinutes(numbers[1][0] / numbers[1][1]);
      coords.setLongitudeSeconds(numbers[2][0] / numbers[2][1]);
    }
  }
  return coords;
}

// Tag names that contain large embedded blobs (base64-encoded image data or
// proprietary maker notes). These are useless to Tanuki and bloat the stored
// raw JSON.
const RAW_TAGS_TO_STRIP = new Set([
  'Thumbnail',
  'PreviewImage',
  'MakerNote',
  'Images',
  'Exif IFD Pointer',
  'GPS Info IFD Pointer',
  'Interoperability IFD Pointer'
]);

export function stripRawImageTags(tags: any): object {
  const out: any = {};
  for (const [key, value] of Object.entries(tags)) {
    if (RAW_TAGS_TO_STRIP.has(key)) continue;
    out[key] = value;
  }
  return out;
}

/**
 * Combined output of running ffprobe on a video file.
 */
export type VideoInfo = {
  originalDate: number | null;
  metadata: AssetMetadata;
  raw: object;
};

/**
 * Run ffprobe on the file at the given path and return the parsed JSON.
 * Returns null if ffprobe exits non-zero or the output cannot be parsed.
 *
 * Uses the canonical "dump useful info" invocation: `-show_format` for
 * container info and tags, `-show_streams` for per-stream details. This is
 * the same shape Namazu returns from `GET /metadata/:id` for videos.
 */
export async function runFfprobe(filepath: string): Promise<any | null> {
  try {
    const proc = Bun.spawn(
      [
        'ffprobe',
        '-v',
        'error',
        '-print_format',
        'json',
        '-show_format',
        '-show_streams',
        filepath
      ],
      { stdout: 'pipe', stderr: 'pipe' }
    );
    const stdout = await new Response(proc.stdout).text();
    await proc.exited;
    if (proc.exitCode !== 0) return null;
    return JSON.parse(stdout);
  } catch {
    return null;
  }
}

/**
 * Extract video metadata and the creation_time date from a parsed ffprobe
 * result. The shape of `probe` matches the JSON returned by
 * `ffprobe -print_format json -show_format -show_streams`.
 */
export function parseVideoMetadata(probe: any): {
  metadata: AssetMetadata;
  originalDate: number | null;
} {
  const metadata = new AssetMetadata();
  const format = probe?.format ?? null;
  const streams: any[] = Array.isArray(probe?.streams) ? probe.streams : [];
  const videoStream = streams.find((s) => s.codec_type === 'video') ?? null;

  // duration: prefer the format-level value, fall back to the stream's.
  if (format?.duration) {
    const d = Number.parseFloat(format.duration);
    if (!Number.isNaN(d)) metadata.duration = d;
  }
  if (metadata.duration === null && videoStream?.duration) {
    const d = Number.parseFloat(videoStream.duration);
    if (!Number.isNaN(d)) metadata.duration = d;
  }

  if (videoStream) {
    metadata.videoCodec = videoStream.codec_name ?? null;
    metadata.frameRate = parseRational(videoStream.r_frame_rate);
    let w = typeof videoStream.width === 'number' ? videoStream.width : null;
    let h = typeof videoStream.height === 'number' ? videoStream.height : null;
    if (w !== null && h !== null && isSideways(videoStream)) {
      [w, h] = [h, w];
    }
    metadata.displayWidth = w;
    metadata.displayHeight = h;
  }

  let originalDate: number | null = null;
  const ct = format?.tags?.creation_time ?? videoStream?.tags?.creation_time;
  if (typeof ct === 'string') {
    const parsed = Date.parse(ct);
    if (!Number.isNaN(parsed)) originalDate = parsed;
  }

  metadata.raw = probe;
  return { metadata, originalDate };
}

function parseRational(value: any): number | null {
  if (typeof value !== 'string') return null;
  const parts = value.split('/');
  if (parts.length !== 2) return null;
  const num = Number.parseFloat(parts[0] || '');
  const den = Number.parseFloat(parts[1] || '');
  if (Number.isNaN(num) || Number.isNaN(den) || den === 0) return null;
  return num / den;
}

function isSideways(stream: any): boolean {
  // Modern ffprobe reports rotation in side_data_list entries of type
  // "Display Matrix". Older versions used stream.tags.rotate. Accept either.
  const sideData: any[] = Array.isArray(stream.side_data_list)
    ? stream.side_data_list
    : [];
  for (const entry of sideData) {
    if (typeof entry.rotation === 'number') {
      const abs = Math.abs(entry.rotation);
      if (abs === 90 || abs === 270) return true;
    }
  }
  const tagRotate = stream.tags?.rotate;
  if (tagRotate !== undefined) {
    const r = Number.parseInt(String(tagRotate), 10);
    if (!Number.isNaN(r)) {
      const abs = Math.abs(r) % 360;
      if (abs === 90 || abs === 270) return true;
    }
  }
  return false;
}

/**
 * Combined ffprobe + metadata extraction for a video file at a local path.
 * Returns null if ffprobe failed.
 */
export async function extractVideoInfo(
  filepath: string
): Promise<VideoInfo | null> {
  const probe = await runFfprobe(filepath);
  if (probe === null) return null;
  const { metadata, originalDate } = parseVideoMetadata(probe);
  return { metadata, originalDate, raw: probe };
}

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
  switch (field) {
    case SortField.Date: {
      if (order === SortOrder.Ascending) {
        results.sort((a, b) => a.datetime.getTime() - b.datetime.getTime());
      } else {
        results.sort((a, b) => b.datetime.getTime() - a.datetime.getTime());
      }

      break;
    }
    case SortField.Identifier: {
      if (order === SortOrder.Ascending) {
        results.sort((a, b) => a.assetId.localeCompare(b.assetId));
      } else {
        results.sort((a, b) => b.assetId.localeCompare(a.assetId));
      }

      break;
    }
    case SortField.Filename: {
      if (order === SortOrder.Ascending) {
        results.sort((a, b) => a.filename.localeCompare(b.filename));
      } else {
        results.sort((a, b) => b.filename.localeCompare(a.filename));
      }

      break;
    }
    case SortField.MediaType: {
      if (order === SortOrder.Ascending) {
        results.sort((a, b) => a.mediaType.localeCompare(b.mediaType));
      } else {
        results.sort((a, b) => b.mediaType.localeCompare(a.mediaType));
      }

      break;
    }
    // No default
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
        if (input.label.length === 0) {
          outgoing.label = null;
        } else {
          outgoing.label = input.label;
        }
      }
      // set or clear the city field
      if (input.city !== null) {
        if (input.city.length === 0) {
          outgoing.city = null;
        } else {
          outgoing.city = input.city;
        }
      }
      // set or clear the region field
      if (input.region !== null) {
        if (input.region.length === 0) {
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
          ident += ch;
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
      ident += ch;
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
