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
    if (city.length > 0) {
      ret.city = city;
    }
    if (region.length > 0) {
      ret.region = region;
    }
    return ret;
  }

  /** Build a Location from exactly the input values without any processing. */
  static fromRaw(label: string | null, city: string | null, region: string | null): Location {
    const ret = new Location(label || '');
    ret.label = label;
    ret.city = city;
    ret.region = region;
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
   * Format a Location field as a string.
   */
  toString(): string {
    if (this.label && this.city && this.region) {
      return `${this.label}; ${this.city}, ${this.region}`;
    } else if (this.city && this.region) {
      return `${this.city}, ${this.region}`;
    } else if (this.label && this.city) {
      return `${this.label}; ${this.city}`;
    } else if (this.label && this.region) {
      return `${this.label}; ${this.region}`;
    } else if (this.label) {
      return this.label;
    } else if (this.city) {
      return this.city;
    } else if (this.region) {
      return this.region;
    }
    return '';
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

type LatitudeRef = 'N' | 'S';
type LongitudeRef = 'E' | 'W';

/**
 * GPS coordinates as read from an image or other sources.
 */
class Coordinates {
  latitudeRef: LatitudeRef;
  latitude: [number, number, number];
  longitudeRef: LongitudeRef;
  longitude: [number, number, number];

  constructor(
    latitudeRef: LatitudeRef,
    latitude: [number, number, number],
    longitudeRef: LongitudeRef,
    longitude: [number, number, number]
  ) {
    this.latitudeRef = latitudeRef;
    this.latitude = latitude;
    this.longitudeRef = longitudeRef;
    this.longitude = longitude;
  }

  /**
   * Return the coordinates as a pair of signed decimals, as expected by some
   * reverse geocoding sytems.
   */
  intoDecimals(): [number, number] {
    const lat = this.latitude[0] + this.latitude[1] / 60 + this.latitude[2] / 3600;
    const long = this.longitude[0] + this.longitude[1] / 60 + this.longitude[2] / 3600;
    const latSign = this.latitudeRef === 'N' ? 1 : -1;
    const longSign = this.longitudeRef === 'E' ? 1 : -1;
    return [latSign * lat, longSign * long];
  }

  setLatitudeRef(latitudeRef: LatitudeRef): Coordinates {
    this.latitudeRef = latitudeRef;
    return this;
  }

  setLatitudeDegrees(degrees: number): Coordinates {
    this.latitude[0] = degrees;
    return this;
  }

  setLatitudeMinutes(minutes: number): Coordinates {
    this.latitude[1] = minutes;
    return this;
  }

  setLatitudeSeconds(seconds: number): Coordinates {
    this.latitude[2] = seconds;
    return this;
  }

  setLongitudeRef(longitudeRef: LongitudeRef): Coordinates {
    this.longitudeRef = longitudeRef;
    return this;
  }

  setLongitudeDegrees(degrees: number): Coordinates {
    this.longitude[0] = degrees;
    return this;
  }

  setLongitudeMinutes(minutes: number): Coordinates {
    this.longitude[1] = minutes;
    return this;
  }

  setLongitudeSeconds(seconds: number): Coordinates {
    this.longitude[2] = seconds;
    return this;
  }
}

/**
 * Location information returned from a reverse geocoding service.
 */
class Geocoded {
  /** Name of the city (locality). */
  city: string | null;
  /** Name of the region (administrative_area_level_1). */
  region: string | null;
  /** Name of the country (country). */
  country: string | null;

  constructor(city: string | null, region: string | null, country: string | null) {
    this.city = city;
    this.region = region;
    this.country = country;
  }
}

export { Coordinates, Geocoded, Location };
