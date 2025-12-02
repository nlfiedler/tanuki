//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, test } from "bun:test";
// prepare the test environment as early as possible
import 'tanuki/test/env.ts';
import { Coordinates } from 'tanuki/server/domain/entities/Location.ts';
import { EnvSettingsRepository } from 'tanuki/server/data/repositories/EnvSettingsRepository.ts';
import { GoogleLocationRepository } from 'tanuki/server/data/repositories/GoogleLocationRepository.ts';

describe('GoogleMaps reverse geocoding', function () {
  test('should raise error for bogus API key', async function () {
    // arrange
    const settingsRepository = new EnvSettingsRepository();
    settingsRepository.set('GOOGLE_MAPS_API_KEY', 'not-a-valid-key');
    const sut = new GoogleLocationRepository({ settingsRepository });
    // act
    const coords = new Coordinates('N', [1, 1, 1], 'E', [1, 1, 1]);
    try {
      await sut.findLocation(coords);
    } catch (err: any) {
      // assert
      expect(err.message).toEqual('The provided API key is invalid.');
    }
  });

  test('should find the address for valid coordinates', async function () {
    // arrange
    const settingsRepository = new EnvSettingsRepository();
    const sut = new GoogleLocationRepository({ settingsRepository });
    // act
    const coords = new Coordinates('N', [34, 37, 17], 'E', [135, 35, 21]);
    const actual = await sut.findLocation(coords);
    // assert
    expect(actual?.city).toEqual('Yao');
    expect(actual?.region).toEqual('Osaka');
    expect(actual?.country).toEqual('Japan');
  });

  test('should return null for bogus coordinates', async function () {
    // arrange
    const settingsRepository = new EnvSettingsRepository();
    const sut = new GoogleLocationRepository({ settingsRepository });
    // act
    const coords = new Coordinates('N', [1, 1, 1], 'E', [1, 1, 1]);
    const actual = await sut.findLocation(coords);
    // assert
    expect(actual).toBeNull();
  });
});
