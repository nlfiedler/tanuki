//
// Copyright (c) 2025 Nathan Fiedler
//
import {
  Coordinates,
  Geocoded
} from 'tanuki/server/domain/entities/location.ts';

/**
 * Repository for finding locations.
 */
interface LocationRepository {
  /**
   * Attempt to find an address for the given GPS coordinates. If no suitable
   * address can be found (e.g. the North pole), then `null` is returned.
   *
   * @param coords GPS coordinates to be located.
   * @returns the address of the coordinates, or null if not found.
   */
  findLocation(coords: Coordinates): Promise<Geocoded | null>;
}

export { type LocationRepository };
