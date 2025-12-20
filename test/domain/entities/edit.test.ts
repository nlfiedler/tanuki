//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, test } from 'bun:test';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { Location } from 'tanuki/server/domain/entities/location.ts';
import * as ops from 'tanuki/server/domain/entities/edit.ts';

describe('Asset operations', function () {
  test('should add a tag to an asset', function () {
    // arrange
    const asset = new Asset('abc123');
    const op = new ops.TagAdd('dog');
    // act
    const modded = op.perform(asset);
    // assert
    expect(modded).toBeTrue();
    expect(asset.tags).toHaveLength(1);
    expect(asset.tags[0]).toEqual('dog');
  });

  test('should not add an already existing tag', function () {
    // arrange
    const asset = new Asset('abc123').setTags(['cat', 'dog']);
    const op = new ops.TagAdd('dog');
    // act
    const modded = op.perform(asset);
    // assert
    expect(modded).toBeFalse();
    expect(asset.tags).toHaveLength(2);
    expect(asset.tags[0]).toEqual('cat');
    expect(asset.tags[1]).toEqual('dog');
  });

  test('should remove a tag from an asset', function () {
    // arrange
    const asset = new Asset('abc123').setTags(['cat', 'dog']);
    const op = new ops.TagRemove('dog');
    // act
    const modded = op.perform(asset);
    // assert
    expect(modded).toBeTrue();
    expect(asset.tags).toHaveLength(1);
    expect(asset.tags[0]).toEqual('cat');
  });

  test('should not remove a tag that does not exist', function () {
    // arrange
    const asset = new Asset('abc123').setTags(['cat', 'dog']);
    const op = new ops.TagRemove('fluffy');
    // act
    const modded = op.perform(asset);
    // assert
    expect(modded).toBeFalse();
    expect(asset.tags).toHaveLength(2);
    expect(asset.tags[0]).toEqual('cat');
    expect(asset.tags[1]).toEqual('dog');
  });

  test('should clear location label of an asset', function () {
    // arrange
    const asset = new Asset('abc123').setLocation(Location.parse('beach; Oahu, Hawaii'));
    const op = new ops.LocationClearField(ops.LocationField.Label);
    // act
    const modded = op.perform(asset);
    // assert
    expect(modded).toBeTrue();
    expect(asset.location?.label).toBeNull();
    expect(asset.location?.city).toEqual('Oahu');
    expect(asset.location?.region).toEqual('Hawaii');
  });

  test('should clear location city of an asset', function () {
    // arrange
    const asset = new Asset('abc123').setLocation(Location.parse('beach; Oahu, Hawaii'));
    const op = new ops.LocationClearField(ops.LocationField.City);
    // act
    const modded = op.perform(asset);
    // assert
    expect(modded).toBeTrue();
    expect(asset.location?.label).toEqual('beach');
    expect(asset.location?.city).toBeNull();
    expect(asset.location?.region).toEqual('Hawaii');
  });

  test('should clear location region of an asset', function () {
    // arrange
    const asset = new Asset('abc123').setLocation(Location.parse('beach; Oahu, Hawaii'));
    const op = new ops.LocationClearField(ops.LocationField.Region);
    // act
    const modded = op.perform(asset);
    // assert
    expect(modded).toBeTrue();
    expect(asset.location?.label).toEqual('beach');
    expect(asset.location?.city).toEqual('Oahu');
    expect(asset.location?.region).toBeNull();
  });

  test('should set location label of an asset', function () {
    // arrange
    const asset = new Asset('abc123').setLocation(Location.parse('beach; Oahu, Hawaii'));
    const op = new ops.LocationSetField(ops.LocationField.Label, 'lagoon');
    // act
    const modded = op.perform(asset);
    // assert
    expect(modded).toBeTrue();
    expect(asset.location?.label).toEqual('lagoon');
    expect(asset.location?.city).toEqual('Oahu');
    expect(asset.location?.region).toEqual('Hawaii');
  });

  test('should set location city of an asset', function () {
    // arrange
    const asset = new Asset('abc123').setLocation(Location.parse('beach; Oahu, Hawaii'));
    const op = new ops.LocationSetField(ops.LocationField.City, 'Honolulu');
    // act
    const modded = op.perform(asset);
    // assert
    expect(modded).toBeTrue();
    expect(asset.location?.label).toEqual('beach');
    expect(asset.location?.city).toEqual('Honolulu');
    expect(asset.location?.region).toEqual('Hawaii');
  });

  test('should set location region of an asset', function () {
    // arrange
    const asset = new Asset('abc123').setLocation(Location.parse('beach; Oahu, Hawaii'));
    const op = new ops.LocationSetField(ops.LocationField.Region, 'HI');
    // act
    const modded = op.perform(asset);
    // assert
    expect(modded).toBeTrue();
    expect(asset.location?.label).toEqual('beach');
    expect(asset.location?.city).toEqual('Oahu');
    expect(asset.location?.region).toEqual('HI');
  });

  test('should clear user date-time of an asset', function () {
    // arrange
    const asset = new Asset('abc123').setUserDate(new Date(2018, 5, 9, 12, 0));
    const op = new ops.DatetimeClear();
    // act
    const modded = op.perform(asset);
    // assert
    expect(modded).toBeTrue();
    expect(asset.userDate).toBeNull();
  });

  test('should set user date-time of an asset', function () {
    // arrange
    const asset = new Asset('abc123').setUserDate(new Date(2018, 5, 9, 12, 0));
    const op = new ops.DatetimeSet(new Date(2003, 7, 30, 12, 0));
    // act
    const modded = op.perform(asset);
    // assert
    expect(modded).toBeTrue();
    expect(asset.userDate?.getFullYear()).toEqual(2003);
  });

  test('should add days to date-time of an asset', function () {
    // arrange
    const asset = new Asset('abc123').setUserDate(new Date(2018, 5, 9, 12, 0));
    const op = new ops.DatetimeAddDays(5);
    // act
    const modded = op.perform(asset);
    // assert
    expect(modded).toBeTrue();
    expect(asset.userDate?.getFullYear()).toEqual(2018);
    expect(asset.userDate?.getMonth()).toEqual(5);
    expect(asset.userDate?.getDate()).toEqual(14);
  });

  test('should subtract days from date-time of an asset', function () {
    // arrange
    const asset = new Asset('abc123').setUserDate(new Date(2018, 5, 9, 12, 0));
    const op = new ops.DatetimeSubDays(5);
    // act
    const modded = op.perform(asset);
    // assert
    expect(modded).toBeTrue();
    expect(asset.userDate?.getFullYear()).toEqual(2018);
    expect(asset.userDate?.getMonth()).toEqual(5);
    expect(asset.userDate?.getDate()).toEqual(4);
  });

  test('should add many days to date-time of an asset', function () {
    // arrange
    const asset = new Asset('abc123').setUserDate(new Date(2018, 5, 9, 12, 0));
    const op = new ops.DatetimeAddDays(365);
    // act
    const modded = op.perform(asset);
    // assert
    expect(modded).toBeTrue();
    expect(asset.userDate?.getFullYear()).toEqual(2019);
    expect(asset.userDate?.getMonth()).toEqual(5);
    expect(asset.userDate?.getDate()).toEqual(9);
  });

  test('should subtract many days from date-time of an asset', function () {
    // arrange
    const asset = new Asset('abc123').setUserDate(new Date(2018, 5, 9, 12, 0));
    const op = new ops.DatetimeSubDays(365);
    // act
    const modded = op.perform(asset);
    // assert
    expect(modded).toBeTrue();
    expect(asset.userDate?.getFullYear()).toEqual(2017);
    expect(asset.userDate?.getMonth()).toEqual(5);
    expect(asset.userDate?.getDate()).toEqual(9);
  });
});
