//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';

type LocationValues = {
  labels: string[];
  cities: string[];
  regions: string[];
};

export default ({
  recordRepository
}: {
  recordRepository: RecordRepository;
}) => {
  assert.ok(recordRepository, 'record repository must be defined');
  /**
   * Return the unique location parts for each field -- all location labels, all
   * location cities, and all location regions.
   *
   * @returns object with lists of unique location values.
   */
  return async (): Promise<LocationValues> => {
    const locations = await recordRepository.rawLocations();
    const labels = new Set<string>();
    const cities = new Set<string>();
    const regions = new Set<string>();
    for (const entry of locations) {
      if (entry.label) {
        labels.add(entry.label);
      }
      if (entry.city) {
        cities.add(entry.city);
      }
      if (entry.region) {
        regions.add(entry.region);
      }
    }
    return {
      labels: Array.from(labels),
      cities: Array.from(cities),
      regions: Array.from(regions)
    };
  };
};
