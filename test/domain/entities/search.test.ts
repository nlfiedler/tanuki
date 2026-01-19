//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, test } from 'bun:test';
import { SearchParams } from 'tanuki/server/domain/entities/search.ts';

describe('SearchParams entity', function () {
  test('should convert parameters to a string', function () {
    expect(new SearchParams().addTag('dog').toString()).toEqual('tag:dog');

    expect(new SearchParams().setTags(['cat', 'dog']).toString()).toEqual(
      'tag:cat tag:dog'
    );

    expect(
      new SearchParams().addTag('dog').addLocation('beach').toString()
    ).toEqual('tag:dog loc:beach');

    expect(
      new SearchParams().addTag('dog').setMediaType('image/jpeg').toString()
    ).toEqual('tag:dog is:image format:jpeg');

    expect(
      new SearchParams()
        .addTag('dog')
        .setBeforeDate(new Date(2018, 5, 18, 12, 0))
        .toString()
    ).toEqual('tag:dog before:2018-06-18T12:00');

    expect(
      new SearchParams()
        .addTag('dog')
        .setAfterDate(new Date(2018, 5, 18, 12, 0))
        .toString()
    ).toEqual('tag:dog after:2018-06-18T12:00');

    expect(
      new SearchParams()
        .setTags(['cat', 'dog'])
        .addLocation('beach')
        .setMediaType('image/jpeg')
        .setBeforeDate(new Date(2017, 5, 18, 12, 0))
        .setAfterDate(new Date(2018, 5, 18, 12, 0))
        .toString()
    ).toEqual(
      'tag:cat tag:dog loc:beach is:image format:jpeg before:2017-06-18T12:00 after:2018-06-18T12:00'
    );

    expect(
      new SearchParams()
        .setMediaType('image/jpeg')
        .setBeforeDate(new Date(2017, 5, 18, 12, 0))
        .setAfterDate(new Date(2018, 5, 18, 12, 0))
        .toString()
    ).toEqual(
      'is:image format:jpeg before:2017-06-18T12:00 after:2018-06-18T12:00'
    );

    expect(
      new SearchParams().setAfterDate(new Date(2018, 5, 18, 12, 0)).toString()
    ).toEqual('after:2018-06-18T12:00');
  });
});
