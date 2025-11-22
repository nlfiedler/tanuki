//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, test } from "bun:test";
import { EnvSettingsRepository } from 'tanuki/server/data/repositories/EnvSettingsRepository.ts';

describe('EnvSettingsRepository', function () {
  const sut = new EnvSettingsRepository();

  test('should return string or undefined from get()', function () {
    process.env['ESR_SETTING_VALUE'] = 'a_value';
    expect(sut.get('ESR_SETTING_VALUE')).toEqual('a_value');
    expect(sut.get('ESR_SETTING_UNDEFINED')).toBeUndefined();
  });

  test('should return true or false from has()', function () {
    process.env['ESR_SETTING_VALUE'] = 'a_value';
    expect(sut.has('ESR_SETTING_VALUE')).toBeTrue();
    expect(sut.has('NO_SUCH_SETTING_BY_THAT_NAME')).toBeFalse();
  });

  test('should return true or false from getBool()', function () {
    process.env['ESR_SETTING_TRUE'] = 'true';
    process.env['ESR_SETTING_NONE'] = 'none';
    process.env['ESR_SETTING_FALSE'] = 'false';
    expect(sut.getBool('ESR_SETTING_TRUE')).toBeTrue();
    expect(sut.getBool('ESR_SETTING_NONE')).toBeFalse();
    expect(sut.getBool('ESR_SETTING_FALSE')).toBeFalse();
    expect(sut.getBool('ESR_SETTING_UNDEFINED')).toBeFalse();
  });

  test('should return value or fallback from getInt()', function () {
    process.env['ESR_SETTING_3000'] = '3000';
    expect(sut.getInt('ESR_SETTING_3000', 100)).toEqual(3000);
    expect(sut.getInt('ESR_SETTING_NONE', 101)).toEqual(101);
  });

  test('should allow changing values', function () {
    sut.set('ESR_TEST_BOOL', 'true');
    expect(sut.getBool('ESR_TEST_BOOL')).toBeTrue();
    sut.set('ESR_TEST_INT', '123');
    expect(sut.get('ESR_TEST_INT')).toEqual('123');
    sut.set('ESR_TEST_STR', 'abc');
    expect(sut.get('ESR_TEST_STR')).toEqual('abc');
  });

  test('should shadow values in process environment', function () {
    process.env['ESR_SETTING_VALUE'] = 'a_value';
    expect(sut.get('ESR_SETTING_VALUE')).toEqual('a_value');
    sut.set('ESR_SETTING_VALUE', 'new_value');
    expect(sut.get('ESR_SETTING_VALUE')).toEqual('new_value');
  });
});
