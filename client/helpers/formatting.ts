//
// Copyright (c) 2025 Nathan Fiedler
//
import type { Location } from 'tanuki/generated/graphql.ts';

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
