//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, test } from 'bun:test';
import {
  Coordinates,
  Location
} from 'tanuki/server/domain/entities/Location.ts';

describe('Location entity', function () {
  test('should indicate if it has values or not', function () {
    const constructorEmpty = new Location('');
    expect(constructorEmpty.hasValues()).toBeFalse();

    const withpartsEmpty = Location.fromParts('', '', '');
    expect(withpartsEmpty.hasValues()).toBeFalse();

    const hasLabel = new Location('label');
    expect(hasLabel.hasValues()).toBeTrue();

    const hasCity = Location.fromParts('', 'city', '');
    expect(hasCity.hasValues()).toBeTrue();

    const hasRegion = Location.fromParts('', '', 'region');
    expect(hasRegion.hasValues()).toBeTrue();
  });

  test('should construct from raw values', function () {
    const allNulls = Location.fromRaw(null, null, null);
    expect(allNulls.hasValues()).toBeFalse();

    const allBlanks = Location.fromRaw('', '', '');
    expect(allBlanks.hasValues()).toBeFalse();

    const hasLabel = Location.fromRaw('label', null, null);
    expect(hasLabel.hasValues()).toBeTrue();

    const hasCity = Location.fromRaw(null, 'city', null);
    expect(hasCity.hasValues()).toBeTrue();

    const hasRegion = Location.fromRaw(null, null, 'region');
    expect(hasRegion.hasValues()).toBeTrue();

    const hasAll = Location.fromRaw('label', 'city', 'region');
    expect(hasAll.hasValues()).toBeTrue();
  });

  test('should parse a string into a Location', function () {
    // emtpy string
    expect(Location.parse('').label).toBeNull();

    // no separators
    expect(Location.parse('classical garden').label).toEqual(
      'classical garden'
    );

    // all 3 parts
    const cgpo = Location.parse('classical garden ; Portland , Oregon');
    expect(cgpo.label).toEqual('classical garden');
    expect(cgpo.city).toEqual('Portland');
    expect(cgpo.region).toEqual('Oregon');

    // label and city
    const ttc = Location.parse('theater ; The City');
    expect(ttc.label).toEqual('theater');
    expect(ttc.city).toEqual('The City');
    expect(ttc.region).toBeNull();

    // city and region
    const kkh = Location.parse('Kailua-Kona, Hawaii');
    expect(kkh.label).toBeNull();
    expect(kkh.city).toEqual('Kailua-Kona');
    expect(kkh.region).toEqual('Hawaii');

    // multiple semi-colons is invalid, converts to label only
    expect(Location.parse('too ; many ; parts').label).toEqual(
      'too ; many ; parts'
    );

    // multiple commas is invalid, converts to label only
    expect(Location.parse('too , many , parts').label).toEqual(
      'too , many , parts'
    );

    // one semicolon but multiple commas is also invalid
    expect(Location.parse('label; too, many, parts').label).toEqual(
      'label; too, many, parts'
    );
  });

  test('should match query to location parts', function () {
    const brazil = Location.parse('beach; São Paulo, State of São Paulo');
    expect(brazil.partialMatch('beach')).toBeTrue();
    expect(brazil.partialMatch('são paulo')).toBeTrue();
    expect(brazil.partialMatch('berkeley')).toBeFalse();

    const france = Location.parse('Paris, France');
    expect(france.partialMatch('paris')).toBeTrue();
    expect(france.partialMatch('france')).toBeTrue();
    expect(france.partialMatch('texas')).toBeFalse();

    const oregon = Location.fromParts('', '', 'Oregon');
    expect(oregon.partialMatch('oregon')).toBeTrue();
    expect(oregon.partialMatch('OREGON')).toBeFalse();
  });
});

describe('Coordinates entity', function () {
  test('should return decimals from coordinates', function () {
    const nw = new Coordinates('N', [34, 37, 17], 'W', [135, 35, 21]);
    expect(nw.intoDecimals()).toEqual([34.62138888888889, -135.58916666666667]);

    const se = new Coordinates('S', [34, 37, 17], 'E', [135, 35, 21]);
    expect(se.intoDecimals()).toEqual([-34.62138888888889, 135.58916666666667]);
  });
});
