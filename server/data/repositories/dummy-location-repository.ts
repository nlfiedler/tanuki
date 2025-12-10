//
// Copyright (c) 2025 Nathan Fiedler
//
import {
  Coordinates,
  Geocoded
} from 'tanuki/server/domain/entities/location.ts';
import { type LocationRepository } from 'tanuki/server/domain/repositories/location-repository.ts';

/**
 * Repository for finding locations.
 */
class DummyLocationRepository implements LocationRepository {
  /**
   * Always returns null.
   */
  findLocation(_coords: Coordinates): Promise<Geocoded | null> {
    return Promise.resolve(null);
  }
}

export { DummyLocationRepository };
