//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, test } from 'bun:test';
import { MapSettingsRepository } from 'tanuki/server/data/repositories/map-settings-repository.ts';

describe('MapSettingsRepository', function () {
  test('should return string or undefined from get()', function () {
    const sut = new MapSettingsRepository();
    sut.set('SETTING_VALUE', 'a_value');
    expect(sut.get('SETTING_VALUE')).toEqual('a_value');
    expect(sut.get('SETTING_UNDEFINED')).toBeUndefined();
  });

  test('should return true or false from has()', function () {
    const sut = new MapSettingsRepository();
    sut.set('SETTING_VALUE', 'a_value');
    expect(sut.has('SETTING_VALUE')).toBeTrue();
    expect(sut.has('NO_SUCH_SETTING_BY_THAT_NAME')).toBeFalse();
  });

  test('should return true or false from getBool()', function () {
    const sut = new MapSettingsRepository();
    sut.set('SETTING_TRUE', 'true');
    sut.set('SETTING_NONE', 'none');
    sut.set('SETTING_FALSE', 'false');
    expect(sut.getBool('SETTING_TRUE')).toBeTrue();
    expect(sut.getBool('SETTING_NONE')).toBeFalse();
    expect(sut.getBool('SETTING_FALSE')).toBeFalse();
    expect(sut.getBool('SETTING_UNDEFINED')).toBeFalse();
  });

  test('should return value or fallback from getInt()', function () {
    const sut = new MapSettingsRepository();
    sut.set('SETTING_3000', '3000');
    sut.set('SETTING_NONE', 'none');
    expect(sut.getInt('SETTING_3000', 100)).toEqual(3000);
    expect(sut.getInt('SETTING_NONE', 101)).toEqual(101);
  });

  test('should allow changing values', function () {
    const sut = new MapSettingsRepository();
    sut.set('HAS_TEST_BOOL', 'true');
    expect(sut.getBool('HAS_TEST_BOOL')).toBeTrue();
    sut.set('HAS_TEST_INT', '123');
    expect(sut.get('HAS_TEST_INT')).toEqual('123');
    sut.set('HAS_TEST_STR', 'abc');
    expect(sut.get('HAS_TEST_STR')).toEqual('abc');
  });
});
