//
// Copyright (c) 2025 Nathan Fiedler
//

/**
 * Location information regarding an asset.
 */
class Location {
  /** User-defined label describing the location. */
  label: string | null;
  /** Name of the city associated with this location. */
  city: string | null;
  /** Name of the region (state, province) associated with this location. */
  region: string | null;

  constructor(label: string) {
    this.label = label.length > 0 ? label : null;
    this.city = null;
    this.region = null;
  }

  /**
   * Construct a Location using all of the parts given. If any parts are
   * empty, then the corresponding field will be null.
   *
   * @param label - user-defined label for this location.
   * @param city - value for the city.
   * @param region - value for the region (state, province).
   */
  static fromParts(label: string, city: string, region: string): Location {
    const ret = new Location(label);
    if (label.length == 0) {
      ret.label = null;
    }
    if (city.length > 0) {
      ret.city = city;
    }
    if (region.length > 0) {
      ret.region = region;
    }
    return ret;
  }

  /**
   * Parse the string into a location. Labels are separated by a semicolon
   * while city and region are separated by a comma. If there are no
   * separators, the input is treated as a label. Label, if present, comes
   * before city and region. If there is a semicolon but no comma, then the
   * second part is treated as the city. If there are too many semicolons or
   * commas, the input is treated as a label.
   * 
   * @param s - string input to be parsed.
   * @returns Location with fields appropriately populated.
   */
  static parse(s: string): Location {
    //
    // possible valid inputs:
    //
    // label
    // label; city
    // label; city, region
    // city, region
    //
    if (s.length === 0) {
      return new Location('');
    } else if (s.includes(';')) {
      const label_tail = s.split(';');
      if (label_tail.length == 2) {
        if (label_tail[1]?.includes(',')) {
          // label; city, region
          const city_region = label_tail[1]?.split(',');
          if (city_region.length == 2) {
            return Location.fromParts(
              label_tail[0]?.trim() || '',
              city_region[0]?.trim() || '',
              city_region[1]?.trim() || '',
            );
          }
        } else {
          // label; city
          return Location.fromParts(
            label_tail[0]?.trim() || '',
            label_tail[1]?.trim() || '',
            '',
          );
        }
      }
    } else if (s.includes(',')) {
      const city_region = s.split(',');
      if (city_region.length == 2) {
        // city, region
        return Location.fromParts(
          '',
          city_region[0]?.trim() || '',
          city_region[1]?.trim() || '',
        );
      }
    }
    // everything else is just a label
    return new Location(s);
  }

  /**
   * Determine if any of the fields has a value.
   *
   * @returns true if any of the fields have a value.
   */
  hasValues(): boolean {
    return this.label !== null || this.city !== null || this.region !== null;
  }

  /**
   * Test if any part of this location matches the query. The parts of the
   * location will be lowercased and compared to the query as-is.
   */
  partialMatch(query: string): boolean {
    if (this.label) {
      if (this.label.toLowerCase() == query) {
        return true;
      }
    }
    if (this.city) {
      if (this.city.toLowerCase() == query) {
        return true;
      }
    }
    if (this.region) {
      return this.region.toLowerCase() == query;
    }
    return false;
  }
}

export { Location };
