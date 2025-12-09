//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import { Coordinates, Geocoded } from 'tanuki/server/domain/entities/Location.ts';
import { type LocationRepository } from 'tanuki/server/domain/repositories/LocationRepository.ts';
import { type SettingsRepository } from 'tanuki/server/domain/repositories/SettingsRepository.ts';

const GOOGLE_MAPS_URI = 'https://maps.googleapis.com/maps/api/geocode/json';

/**
 * Repository for finding locations using the Google Maps API.
 */
class GoogleLocationRepository implements LocationRepository {
  apiKey: string;

  constructor({ settingsRepository }: { settingsRepository: SettingsRepository; }) {
    this.apiKey = settingsRepository.get('GOOGLE_MAPS_API_KEY');
    assert.ok(this.apiKey, 'missing GOOGLE_MAPS_API_KEY environment variable');
  }

  /**
   * Attempt to find an address for the given GPS coordinates. If no suitable
   * address can be found (e.g. the North pole), then the fields of the result
   * will all be `null`.
   *
   * @param coords GPS coordinates to be located.
   * @returns the address of the coordinates, or null/null/null if not found.
   */
  async findLocation(coords: Coordinates): Promise<Geocoded | null> {
    const url = new URL(GOOGLE_MAPS_URI);
    url.searchParams.append('key', this.apiKey);
    const [lat, long] = coords.intoDecimals();
    url.searchParams.append('latlng', `${lat},${long}`);
    url.searchParams.append('result_type', 'country|administrative_area_level_1|locality');

    let retries = 0;
    while (retries < 10) {
      const response = await fetch(url.toString());
      if (!response.ok) {
        throw new Error('expected 200 response');
      }
      const result = await response.json();
      if (result.status === 'ZERO_RESULTS') {
        return null;
      }
      if (result.status === 'OVER_QUERY_LIMIT') {
        // back off gradually
        retries++;
        await sleep(retries * 100);
      } else if ('error_message' in result) {
        // may happen for an invalid API key among other things
        throw new Error(result.error_message.trim());
      } else {
        // the first entry in the results array is the most specific and the
        // remaining entries increase in scope, we only want the first one
        const location = new Geocoded(null, null, null);
        for (const component of result.results[0].address_components) {
          for (const type_ of component.types) {
            if (type_ === 'locality') {
              location.city = component.long_name;
            } else if (type_ === 'administrative_area_level_1') {
              location.region = component.long_name;
            } else if (type_ === 'country') {
              location.country = component.long_name;
            }
          }
        }
        return location;
      }
    }
    throw new Error('failed after retry');
  }
}

function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export { GoogleLocationRepository };
