//
// Copyright (c) 2025 Nathan Fiedler
//
import type {
  AssetMetadata,
  Location,
  SearchResult
} from 'tanuki/generated/graphql.ts';

/**
 * Format the given date-time as a date string using `toDateString()`.
 */
export function formatDatetime(
  datetime: string | Date | null | undefined
): string {
  if (typeof datetime === 'string') {
    return new Date(datetime).toDateString();
  } else if (datetime) {
    return datetime.toDateString();
  }
  return '';
}

/**
 * Format a Location field as a string.
 */
export function formatLocation(location: Location): string {
  if (location.label && location.city && location.region) {
    return `${location.label}; ${location.city}, ${location.region}`;
  } else if (location.city && location.region) {
    return `${location.city}, ${location.region}`;
  } else if (location.label && location.city) {
    return `${location.label}; ${location.city}`;
  } else if (location.label && location.region) {
    return `${location.label}; ${location.region}`;
  } else if (location.label) {
    return location.label;
  } else if (location.city) {
    return location.city;
  } else if (location.region) {
    return location.region;
  }
  return '';
}

/**
 * Format a SearchResult's "title" line in the style of PhotoPrism's gallery
 * detail view: location label / city / year. Falls back to the filename when
 * no location is known.
 */
export function formatTitle(asset: SearchResult): string {
  const datetime =
    typeof asset.datetime === 'string'
      ? new Date(asset.datetime)
      : asset.datetime;
  const year =
    datetime instanceof Date && !Number.isNaN(datetime.getTime())
      ? String(datetime.getFullYear())
      : null;
  const parts = [
    asset.location?.label || null,
    asset.location?.city || null,
    year
  ].filter(Boolean);
  if (parts.length === 0) return asset.filename;
  return parts.join(' / ');
}

/**
 * Format the camera body / exposure summary line, e.g.
 * "Apple iPhone 15 Pro, ISO 2000, 1/2833". Returns an empty string if no
 * field carries useful data.
 */
export function formatCamera(meta: AssetMetadata | null | undefined): string {
  if (!meta) return '';
  const body = joinNonEmpty([meta.cameraMake, meta.cameraModel], ' ');
  const parts: string[] = [];
  if (body) parts.push(body);
  if (meta.iso != null) parts.push(`ISO ${meta.iso}`);
  if (meta.exposureTime) parts.push(meta.exposureTime);
  return parts.join(', ');
}

/**
 * Format the lens / aperture / focal length line, e.g.
 * "iPhone 15 Pro 6.765mm f/1.78, 24mm".
 */
export function formatLens(meta: AssetMetadata | null | undefined): string {
  if (!meta) return '';
  const lensMake = meta.lensMake === meta.cameraMake ? null : meta.lensMake;
  const lens = joinNonEmpty([lensMake, meta.lensModel], ' ');
  const parts: string[] = [];
  if (lens) parts.push(lens);
  if (meta.fNumber != null) parts.push(`f/${meta.fNumber}`);
  if (meta.focalLength35mm != null) parts.push(`${meta.focalLength35mm}mm`);
  return parts.join(', ');
}

/**
 * Format the "format, dimensions, size" summary, e.g. "JPEG, 2260 × 1695, 1.7 MB".
 * Always returns the upper-cased media subtype if present; appends pieces as
 * the metadata allows.
 */
export function formatFormat(
  mediaType: string,
  meta: AssetMetadata | null | undefined
): string {
  const parts: string[] = [];
  const subtype = mediaType.split('/')[1];
  if (subtype) parts.push(subtype.toUpperCase());
  if (meta?.displayWidth && meta.displayHeight) {
    parts.push(`${meta.displayWidth} × ${meta.displayHeight}`);
  }
  if (meta?.byteLength != null) {
    parts.push(formatFileSize(Number(meta.byteLength)));
  }
  return parts.join(', ');
}

/** Format a byte count as a short human-readable string (e.g. "1.7 MB"). */
export function formatFileSize(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return '';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let value = bytes;
  let i = 0;
  while (value >= 1024 && i < units.length - 1) {
    value /= 1024;
    i++;
  }
  const formatted = i === 0 ? value.toFixed(0) : value.toFixed(1);
  return `${formatted} ${units[i]}`;
}

/**
 * Format a datetime including the day-of-week and the original timezone
 * offset (e.g. "Wed, Oct 23, 2024 at 2:53 PM GMT+2"). When no offset is
 * recorded, render in the local timezone.
 */
export function formatDatetimeWithTZ(
  datetime: string | Date | null | undefined,
  originalDateOffset: string | null | undefined
): string {
  const date =
    typeof datetime === 'string'
      ? new Date(datetime)
      : (datetime instanceof Date
        ? datetime
        : null);
  if (!date || Number.isNaN(date.getTime())) return '';
  // When the source offset is known (e.g. "+09:00"), shift the absolute
  // instant so toLocaleString output reflects the camera's wall-clock time.
  const offsetMinutes = parseOffset(originalDateOffset);
  const shifted =
    offsetMinutes === null
      ? date
      : new Date(date.getTime() + offsetMinutes * 60 * 1000);
  const opts: Intl.DateTimeFormatOptions = {
    weekday: 'short',
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
    timeZone: offsetMinutes === null ? undefined : 'UTC'
  };
  const formatted = shifted.toLocaleString(undefined, opts);
  if (offsetMinutes === null) return formatted;
  return `${formatted} ${formatGmtOffset(offsetMinutes)}`;
}

function joinNonEmpty(
  parts: (string | null | undefined)[],
  sep: string
): string {
  return parts.filter(Boolean).join(sep);
}

function parseOffset(offset: string | null | undefined): number | null {
  if (!offset) return null;
  const match = /^([+-])(\d{2}):?(\d{2})$/.exec(offset);
  if (!match) return null;
  const sign = match[1] === '-' ? -1 : 1;
  const hours = Number(match[2]);
  const minutes = Number(match[3]);
  return sign * (hours * 60 + minutes);
}

function formatGmtOffset(minutes: number): string {
  const sign = minutes >= 0 ? '+' : '-';
  const abs = Math.abs(minutes);
  const hh = Math.floor(abs / 60);
  const mm = abs % 60;
  return mm === 0 ? `GMT${sign}${hh}` : `GMT${sign}${hh}:${String(mm).padStart(2, '0')}`;
}
