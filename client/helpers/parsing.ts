//
// Copyright (c) 2025 Nathan Fiedler
//
import type { Location } from 'tanuki/generated/graphql.ts';

/**
 * Parse a string into a Location record.
 */
export function parseLocation(s: string): Location {
  //
  // possible valid inputs:
  //
  // label
  // label; city
  // label; city, region
  // city, region
  //
  if (s.length === 0) {
    return { label: null, city: null, region: null };
  } else if (s.includes(';')) {
    const label_tail = s.split(';');
    if (label_tail.length == 2) {
      if (label_tail[1]?.includes(',')) {
        // label; city, region
        const city_region = label_tail[1]?.split(',');
        if (city_region.length == 2) {
          return {
            label: label_tail[0]?.trim() || null,
            city: city_region[0]?.trim() || null,
            region: city_region[1]?.trim() || null
          };
        }
      } else {
        // label; city
        return {
          label: label_tail[0]?.trim() || null,
          city: label_tail[1]?.trim() || null,
          region: null
        };
      }
    }
  } else if (s.includes(',')) {
    const city_region = s.split(',');
    if (city_region.length == 2) {
      // city, region
      return {
        label: null,
        city: city_region[0]?.trim() || null,
        region: city_region[1]?.trim() || null
      };
    }
  }
  // everything else is just a label
  return { label: s, city: null, region: null };
}
