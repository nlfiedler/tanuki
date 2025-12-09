//
// Copyright (c) 2025 Nathan Fiedler
//
import crypto from 'node:crypto';
import { describe, expect, test } from 'bun:test';
// prepare the test environment as early as possible
import 'tanuki/test/env.ts';
import { Asset } from 'tanuki/server/domain/entities/Asset.ts';
import { Location } from 'tanuki/server/domain/entities/Location.ts';
import { EnvSettingsRepository } from 'tanuki/server/data/repositories/EnvSettingsRepository.ts';
import { CouchDBRecordRepository } from 'tanuki/server/data/repositories/CouchDBRecordRepository.ts';

describe('CouchDBRecordRepository', function () {
  test('should return zero when database is empty', async function () {
    // arrange
    const settingsRepository = new EnvSettingsRepository();
    const sut = new CouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();
    // act
    const count = await sut.countAssets();
    // assert
    expect(count).toEqual(0);
  });

  test('should store a new document, update, and count', async function () {
    // arrange
    const settingsRepository = new EnvSettingsRepository();
    const sut = new CouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();
    // use a semi-realistic asset identifier with mixed case
    const original = new Asset(
      'MjAxMi8wOC8xNS8wMDB4YWw2czQxYWN0YXY5d2V2Z2VtbXZyYS5qcGc='
    );
    original.setChecksum('sha1-c0ff89017fb951c58aab5e585364a15d359fa2c2');
    original.setUserDate(new Date(2003, 7, 30, 12, 0));
    // act
    await sut.putAsset(original);
    // assert
    const fetched = await sut.getAssetById(original.key);
    expect(fetched).toBeDefined();
    expect(fetched!.key).toEqual(
      'MjAxMi8wOC8xNS8wMDB4YWw2czQxYWN0YXY5d2V2Z2VtbXZyYS5qcGc='
    );
    expect(fetched!.checksum).toEqual(
      'sha1-c0ff89017fb951c58aab5e585364a15d359fa2c2'
    );
    expect(fetched!.userDate?.getFullYear()).toEqual(2003);
    expect(fetched!.location).toBeNull();
    let count = await sut.countAssets();
    expect(count).toEqual(1);

    // update the document via the setters on the Asset entity to ensure the
    // object returned from CouchDB is an Asset entity and not just a plain
    // JavaScript object with the appropriate properties
    fetched
      ?.setTags(['bunny', 'rabbit'])
      .setCaption('playing at the zoo')
      .setLocation(Location.parse('Oakland, CA'));
    await sut.putAsset(fetched!);
    const updated = await sut.getAssetById(original.key);
    expect(updated).toBeDefined();
    expect(updated!.key).toEqual(
      'MjAxMi8wOC8xNS8wMDB4YWw2czQxYWN0YXY5d2V2Z2VtbXZyYS5qcGc='
    );
    expect(updated!.checksum).toEqual(
      'sha1-c0ff89017fb951c58aab5e585364a15d359fa2c2'
    );
    expect(updated!.tags).toHaveLength(2);
    expect(updated!.tags[0]).toEqual('bunny');
    expect(updated!.tags[1]).toEqual('rabbit');
    expect(updated!.caption).toEqual('playing at the zoo');
    expect(updated!.location).toEqual(Location.parse('Oakland, CA'));
    // invoke bestDate() to ensure the object returned is really an Asset entity
    expect(updated!.bestDate().getFullYear()).toEqual(2003);
    expect(fetched!.location?.hasValues()).toBeTrue();
    count = await sut.countAssets();
    expect(count).toEqual(1);
  });

  test('should retrieve document by checksum', async function () {
    // arrange
    const settingsRepository = new EnvSettingsRepository();
    const sut = new CouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();
    const doc = new Asset('eagle');
    doc.checksum = 'sha1-cafebabe';
    // act
    await sut.putAsset(doc);
    // assert
    const asset = await sut.getAssetByDigest(doc.checksum);
    expect(asset).toBeDefined();
    expect(asset!.key).toEqual('eagle');
    expect(asset!.checksum).toEqual(doc.checksum);
    expect(asset!.importDate).toEqual(doc.importDate);
  });

  test('should retrieve tags and their counts', async function () {
    // setup
    const settingsRepository = new EnvSettingsRepository();
    const sut = new CouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    await sut.putAsset(buildBasicAsset('basic123')); // tags: cat, dog
    await sut.putAsset(buildBasicAsset('monday6').setTags(['bird', 'dog']));
    await sut.putAsset(buildBasicAsset('tuesday7').setTags(['CAT', 'mouse']));
    await sut.putAsset(buildBasicAsset('wednesday8').setTags(['Cat', 'bird']));
    await sut.putAsset(buildBasicAsset('thursday9').setTags(['dog', 'mouse']));
    const allTags = await sut.allTags();
    expect(allTags).toHaveLength(4);
    expect(allTags[0]?.label).toEqual('bird');
    expect(allTags[0]?.count).toEqual(2);
    expect(allTags[1]?.label).toEqual('cat');
    expect(allTags[1]?.count).toEqual(3);
    expect(allTags[2]?.label).toEqual('dog');
    expect(allTags[2]?.count).toEqual(3);
    expect(allTags[3]?.label).toEqual('mouse');
    expect(allTags[3]?.count).toEqual(2);
  });

  test('should retrieve locations and their counts', async function () {
    // setup
    const settingsRepository = new EnvSettingsRepository();
    const sut = new CouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    await sut.putAsset(buildBasicAsset('basic123')); // location: hawaii
    await sut.putAsset(
      buildBasicAsset('monday6').setLocation(Location.parse('Paris, France'))
    );
    await sut.putAsset(
      buildBasicAsset('tuesday7').setLocation(Location.parse('Paris, Texas'))
    );
    await sut.putAsset(
      buildBasicAsset('wednesday8').setLocation(Location.parse('Dallas, Texas'))
    );
    await sut.putAsset(
      buildBasicAsset('thursday9').setLocation(Location.parse('Oahu, Hawaii'))
    );
    const allTags = await sut.allLocations();
    expect(allTags).toHaveLength(6);
    expect(allTags[0]?.label).toEqual('dallas');
    expect(allTags[0]?.count).toEqual(1);
    expect(allTags[1]?.label).toEqual('france');
    expect(allTags[1]?.count).toEqual(1);
    expect(allTags[2]?.label).toEqual('hawaii');
    expect(allTags[2]?.count).toEqual(2);
    expect(allTags[3]?.label).toEqual('oahu');
    expect(allTags[3]?.count).toEqual(1);
    expect(allTags[4]?.label).toEqual('paris');
    expect(allTags[4]?.count).toEqual(2);
    expect(allTags[5]?.label).toEqual('texas');
    expect(allTags[5]?.count).toEqual(2);
  });

  test('should retrieve years and their counts', async function () {
    // setup
    const settingsRepository = new EnvSettingsRepository();
    const sut = new CouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    await sut.putAsset(buildBasicAsset('basic113')); // 2018
    await sut.putAsset(
      buildBasicAsset('monday6').setUserDate(new Date(2010, 1, 1, 0, 0))
    );
    await sut.putAsset(
      buildBasicAsset('tuesday7').setUserDate(new Date(2012, 1, 1, 0, 0))
    );
    await sut.putAsset(
      buildBasicAsset('wednesday8').setUserDate(new Date(2012, 1, 1, 0, 0))
    );
    const allYears = await sut.allYears();
    expect(allYears).toHaveLength(3);
    expect(allYears[0]?.label).toEqual('2010');
    expect(allYears[0]?.count).toEqual(1);
    expect(allYears[1]?.label).toEqual('2012');
    expect(allYears[1]?.count).toEqual(2);
    expect(allYears[2]?.label).toEqual('2018');
    expect(allYears[2]?.count).toEqual(1);
  });

  test('should retrieve media types and their counts', async function () {
    // setup
    const settingsRepository = new EnvSettingsRepository();
    const sut = new CouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    await sut.putAsset(buildBasicAsset('basic113'));
    await sut.putAsset(
      buildBasicAsset('monday6')
        .setFilename('img_2345.jpg')
        .setMediaType('image/png')
    );
    await sut.putAsset(
      buildBasicAsset('tuesday7')
        .setFilename('img_3456.jpg')
        .setMediaType('video/mpeg')
    );
    await sut.putAsset(
      buildBasicAsset('wednesday8')
        .setFilename('img_4567.jpg')
        .setMediaType('IMAGE/JPEG')
    );
    const allTypes = await sut.allMediaTypes();
    expect(allTypes).toHaveLength(3);
    expect(allTypes[0]?.label).toEqual('image/jpeg');
    expect(allTypes[0]?.count).toEqual(2);
    expect(allTypes[1]?.label).toEqual('image/png');
    expect(allTypes[1]?.count).toEqual(1);
    expect(allTypes[2]?.label).toEqual('video/mpeg');
    expect(allTypes[2]?.count).toEqual(1);
  });

  test('should retrieve documents by tags', async function () {
    // setup
    const settingsRepository = new EnvSettingsRepository();
    const sut = new CouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    // zero assets
    const zero = await sut.queryByTags(['cAt']);
    expect(zero).toHaveLength(0);

    // one asset
    await sut.putAsset(buildBasicAsset('basic123'));
    const one = await sut.queryByTags(['cAt']);
    expect(one).toHaveLength(1);
    expect(one[0]?.filename).toEqual('img_1234.jpg');

    // multiple assets
    await sut.putAsset(
      buildBasicAsset('monday6')
        .setFilename('img_2345.jpg')
        .setTags(['bird', 'dog'])
    );
    await sut.putAsset(
      buildBasicAsset('tuesday7')
        .setFilename('img_3456.jpg')
        .setUserDate(new Date(2004, 4, 31, 21, 10, 11))
        .setTags(['CAT', 'mouse'])
    );
    await sut.putAsset(
      buildBasicAsset('wednesday8')
        .setFilename('img_4567.jpg')
        .setUserDate(new Date(2007, 4, 31, 21, 10, 11))
        .setTags(['Cat', 'lizard', 'chicken'])
    );
    await sut.putAsset(
      buildBasicAsset('thursday9')
        .setFilename('img_5678.jpg')
        .setTags(['bird', 'dog'])
    );
    await sut.putAsset(
      buildBasicAsset('friday10')
        .setFilename('img_6789.jpg')
        .setTags(['mouse', 'house'])
    );
    const multi = await sut.queryByTags(['cAt']);
    expect(multi).toHaveLength(3);
    expect(multi.every((l) => l.mediaType == 'image/jpeg')).toBeTrue();
    expect(multi.every((l) => l.location!.label == 'hawaii')).toBeTrue();
    expect(
      multi.some(
        (l) =>
          l.assetId == 'basic123' &&
          l.filename == 'img_1234.jpg' &&
          l.datetime.getFullYear() == 2018
      )
    ).toBeTrue();
    expect(
      multi.some(
        (l) =>
          l.assetId == 'tuesday7' &&
          l.filename == 'img_3456.jpg' &&
          l.datetime.getFullYear() == 2004
      )
    ).toBeTrue();
    expect(
      multi.some(
        (l) =>
          l.assetId == 'wednesday8' &&
          l.filename == 'img_4567.jpg' &&
          l.datetime.getFullYear() == 2007
      )
    ).toBeTrue();
  });

  test('should retrieve documents by date', async function () {
    // setup
    const settingsRepository = new EnvSettingsRepository();
    const sut = new CouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();
    const min_utc = new Date(-8640000000000000);
    const year_1918 = new Date(1918, 7, 30, 12, 12, 12);
    const year_1940 = new Date(1940, 7, 30, 12, 12, 12);
    const year_1968 = new Date(1968, 7, 30, 12, 12, 12);
    const year_1971 = new Date(1971, 7, 30, 12, 12, 12);
    const year_1996 = new Date(1996, 7, 30, 12, 12, 12);
    const year_2003 = new Date(2003, 7, 30, 12, 12, 12);
    const year_2008 = new Date(2008, 7, 30, 12, 12, 12);
    const year_2011 = new Date(2011, 7, 30, 12, 12, 12);
    const year_2013 = new Date(2013, 7, 30, 12, 12, 12);
    const year_2019 = new Date(2019, 7, 30, 12, 12, 12);
    const year_2020 = new Date(2020, 7, 30, 12, 12, 12);
    const future_date = new Date();
    future_date.setDate(future_date.getDate() + 28);
    const max_utc = new Date(8640000000000000);

    // zero assets
    expect(await sut.queryBeforeDate(future_date)).toHaveLength(0);
    expect(await sut.queryAfterDate(year_1918)).toHaveLength(0);
    expect(await sut.queryDateRange(year_1918, future_date)).toHaveLength(0);

    // one asset
    await sut.putAsset(buildBasicAsset('year_2018'));
    expect(await sut.queryBeforeDate(year_2011)).toHaveLength(0);
    expect(await sut.queryBeforeDate(year_2019)).toHaveLength(1);
    expect(await sut.queryAfterDate(year_2011)).toHaveLength(1);
    expect(await sut.queryAfterDate(year_2019)).toHaveLength(0);
    expect(await sut.queryDateRange(year_2011, year_2019)).toHaveLength(1);

    // multiple assets; set different date fields to test 'best date' logic
    await sut.putAsset(
      buildBasicAsset('year_1940')
        .setUserDate(year_1940)
        .setImportDate(year_2011)
    );
    await sut.putAsset(buildBasicAsset('year_1996').setOriginalDate(year_1996));
    await sut.putAsset(buildBasicAsset('year_2003').setOriginalDate(year_2003));
    await sut.putAsset(buildBasicAsset('year_2008').setOriginalDate(year_2008));
    await sut.putAsset(
      buildBasicAsset('year_2011')
        .setUserDate(year_2011)
        .setImportDate(year_2019)
    );
    await sut.putAsset(buildBasicAsset('year_2013').setUserDate(year_2013));
    await sut.putAsset(buildBasicAsset('future_date').setUserDate(future_date));

    expect(await sut.queryBeforeDate(year_1918)).toHaveLength(0);
    expect(await sut.queryBeforeDate(max_utc)).toHaveLength(8);
    expect(await sut.queryBeforeDate(min_utc)).toHaveLength(0);

    // just before the epoch
    const before_epoch = await sut.queryBeforeDate(year_1968);
    expect(before_epoch).toHaveLength(1);
    expect(before_epoch[0]?.assetId).toEqual('year_1940');

    // just after the epoch
    const after_epoch = await sut.queryBeforeDate(year_1971);
    expect(after_epoch).toHaveLength(1);
    expect(after_epoch[0]?.assetId).toEqual('year_1940');

    const result_before_2011 = await sut.queryBeforeDate(year_2011);
    expect(result_before_2011).toHaveLength(4);
    expect(result_before_2011.some((l) => l.assetId == 'year_1940')).toBeTrue();
    expect(result_before_2011.some((l) => l.assetId == 'year_1996')).toBeTrue();
    expect(result_before_2011.some((l) => l.assetId == 'year_2003')).toBeTrue();
    expect(result_before_2011.some((l) => l.assetId == 'year_2008')).toBeTrue();

    const result_after_2020 = await sut.queryAfterDate(year_2020);
    expect(result_after_2020).toHaveLength(1);
    expect(result_after_2020[0]?.assetId).toEqual('future_date');

    const result_after_2008 = await sut.queryAfterDate(year_2008);
    expect(result_after_2008).toHaveLength(5);
    expect(result_after_2008.some((l) => l.assetId == 'year_2008')).toBeTrue();
    expect(result_after_2008.some((l) => l.assetId == 'year_2011')).toBeTrue();
    expect(result_after_2008.some((l) => l.assetId == 'year_2013')).toBeTrue();
    expect(result_after_2008.some((l) => l.assetId == 'year_2018')).toBeTrue();
    expect(
      result_after_2008.some((l) => l.assetId == 'future_date')
    ).toBeTrue();

    const result_after_min = await sut.queryAfterDate(min_utc);
    expect(result_after_min).toHaveLength(8);
    const result_after_max = await sut.queryAfterDate(max_utc);
    expect(result_after_max).toHaveLength(0);
    const result_after_1918 = await sut.queryAfterDate(year_1918);
    expect(result_after_1918).toHaveLength(8);

    const between_2011_2019 = await sut.queryDateRange(year_2011, year_2019);
    expect(between_2011_2019).toHaveLength(3);
    expect(between_2011_2019.some((l) => l.assetId == 'year_2011')).toBeTrue();
    expect(between_2011_2019.some((l) => l.assetId == 'year_2013')).toBeTrue();
    expect(between_2011_2019.some((l) => l.assetId == 'year_2018')).toBeTrue();

    const between_min_max = await sut.queryDateRange(min_utc, max_utc);
    expect(between_min_max).toHaveLength(8);

    const between_1918_1968 = await sut.queryDateRange(year_1918, year_1968);
    expect(between_1918_1968).toHaveLength(1);
    expect(between_1918_1968[0]?.assetId).toEqual('year_1940');

    const between_1918_1971 = await sut.queryDateRange(year_1918, year_1971);
    expect(between_1918_1971).toHaveLength(1);
    expect(between_1918_1971[0]?.assetId).toEqual('year_1940');

    const between_1940_2013 = await sut.queryDateRange(year_1940, year_2013);
    expect(between_1940_2013).toHaveLength(5);
    expect(between_1940_2013.some((l) => l.assetId == 'year_1940')).toBeTrue();
    expect(between_1940_2013.some((l) => l.assetId == 'year_1996')).toBeTrue();
    expect(between_1940_2013.some((l) => l.assetId == 'year_2003')).toBeTrue();
    expect(between_1940_2013.some((l) => l.assetId == 'year_2008')).toBeTrue();
    expect(between_1940_2013.some((l) => l.assetId == 'year_2011')).toBeTrue();
  });

  test('should retrieve unique location records', async function () {
    // setup
    const settingsRepository = new EnvSettingsRepository();
    const sut = new CouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    // zero assets
    const zero = await sut.rawLocations();
    expect(zero).toHaveLength(0);

    // one asset
    await sut.putAsset(buildBasicAsset('basic113'));
    const one = await sut.rawLocations();
    expect(one).toHaveLength(1);
    expect(one[0]?.label == 'hawaii');

    // multiple assets
    await sut.putAsset(
      buildBasicAsset('monday6')
        .setFilename('img_2345.jpg')
        .setLocation(Location.parse('Paris, France'))
    );
    await sut.putAsset(
      buildBasicAsset('monday8')
        .setFilename('img_6543.jpg')
        .setLocation(Location.parse('Nice, France'))
    );
    await sut.putAsset(
      buildBasicAsset('tuesday7')
        .setFilename('img_3456.jpg')
        .setLocation(Location.parse('Paris, France'))
    );
    await sut.putAsset(
      buildBasicAsset('wednesday8')
        .setFilename('img_4567.jpg')
        .setLocation(Location.parse('museum; Paris, France'))
    );
    await sut.putAsset(
      buildBasicAsset('thursday9')
        .setFilename('img_5678.jpg')
        .setLocation(Location.fromParts('', 'Oahu', 'Hawaii'))
    );
    await sut.putAsset(
      buildBasicAsset('friday10')
        .setFilename('img_6789.jpg')
        .setLocation(Location.parse('museum'))
    );
    await sut.putAsset(
      buildBasicAsset('friday11')
        .setFilename('img_6879.jpg')
        .setLocation(Location.fromParts('garden', 'Portland', 'OR'))
    );
    const single = await sut.rawLocations();
    expect(single).toHaveLength(7);
    expect(single.some((l) => l.toString() == 'Nice, France')).toBeTrue();
    expect(single.some((l) => l.toString() == 'Oahu, Hawaii')).toBeTrue();
    expect(single.some((l) => l.toString() == 'Paris, France')).toBeTrue();
    expect(
      single.some((l) => l.toString() == 'garden; Portland, OR')
    ).toBeTrue();
    expect(single.some((l) => l.toString() == 'hawaii')).toBeTrue();
    expect(single.some((l) => l.toString() == 'museum')).toBeTrue();
    expect(
      single.some((l) => l.toString() == 'museum; Paris, France')
    ).toBeTrue();
  });

  test('should retrieve documents by location', async function () {
    // setup
    const settingsRepository = new EnvSettingsRepository();
    const sut = new CouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    // zero assets
    const zero = await sut.queryByLocations(['haWAii']);
    expect(zero).toHaveLength(0);

    // one asset
    await sut.putAsset(buildBasicAsset('basic113'));
    const one = await sut.queryByLocations(['haWAii']);
    expect(one).toHaveLength(1);
    expect(one[0]?.assetId == 'basic123');
    expect(one[0]?.mediaType == 'image/jpeg');
    expect(one[0]?.filename == 'img_1234.jpg');
    expect(one[0]?.location?.label == 'hawaii');

    // multiple assets
    await sut.putAsset(
      buildBasicAsset('monday6')
        .setFilename('img_2345.jpg')
        .setLocation(Location.parse('Paris, France'))
    );
    await sut.putAsset(
      buildBasicAsset('monday8')
        .setFilename('img_6543.jpg')
        .setLocation(Location.parse('Nice, France'))
    );
    await sut.putAsset(
      buildBasicAsset('tuesday7')
        .setFilename('img_3456.jpg')
        .setLocation(new Location('london'))
    );
    await sut.putAsset(
      buildBasicAsset('wednesday8')
        .setFilename('img_4567.jpg')
        .setLocation(new Location('seoul'))
    );
    await sut.putAsset(
      buildBasicAsset('thursday9')
        .setFilename('img_5678.jpg')
        .setLocation(Location.fromParts('', 'Oahu', 'Hawaii'))
    );
    await sut.putAsset(
      buildBasicAsset('friday10')
        .setFilename('img_6789.jpg')
        .setLocation(new Location('paris'))
    );
    await sut.putAsset(
      buildBasicAsset('friday11')
        .setFilename('img_6879.jpg')
        .setLocation(Location.fromParts('city center', 'Portland', 'Oregon'))
    );

    // searching with a single location
    const single = await sut.queryByLocations(['hawaii']);
    expect(single).toHaveLength(2);
    expect(single.some((l) => l.assetId == 'basic113')).toBeTrue();
    expect(single.some((l) => l.assetId == 'thursday9')).toBeTrue();

    // searching with multiple locations; test the result's location field to
    // ensure it is a Location entity with the appropriate member functions
    const multiple = await sut.queryByLocations(['hawaii', 'oahu']);
    expect(multiple).toHaveLength(1);
    expect(multiple[0]?.assetId).toEqual('thursday9');
    expect(multiple[0]?.location).toEqual(Location.parse('Oahu, Hawaii'));
    expect(multiple[0]?.location?.partialMatch('hawaii')).toBeTrue();

    // searching location term split from commas
    const france = await sut.queryByLocations(['france']);
    expect(france).toHaveLength(2);
    expect(france.some((l) => l.assetId == 'monday6')).toBeTrue();
    expect(france.some((l) => l.assetId == 'monday8')).toBeTrue();

    // searching location term from region field
    const oregon = await sut.queryByLocations(['oregon']);
    expect(oregon).toHaveLength(1);
    expect(oregon[0]?.assetId).toEqual('friday11');
    expect(oregon[0]?.location).toEqual(
      Location.parse('city center; Portland, Oregon')
    );
    expect(oregon[0]?.location?.partialMatch('portland')).toBeTrue();
  });

  test('should retrieve documents by media type', async function () {
    // setup
    const settingsRepository = new EnvSettingsRepository();
    const sut = new CouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    // zero assets
    expect(await sut.queryByMediaType('image/jpeg')).toHaveLength(0);

    // one asset
    await sut.putAsset(buildBasicAsset('basic113'));
    const one = await sut.queryByMediaType('image/jpeg');
    expect(one).toHaveLength(1);
    expect(one[0]?.assetId).toEqual('basic113');

    // multiple assets
    await sut.putAsset(
      buildBasicAsset('monday6')
        .setFilename('img_2345.jpg')
        .setMediaType('image/png')
    );
    await sut.putAsset(
      buildBasicAsset('tuesday7')
        .setFilename('img_3456.jpg')
        .setMediaType('video/mpeg')
    );
    await sut.putAsset(
      buildBasicAsset('wednesday8')
        .setFilename('img_4567.jpg')
        .setMediaType('IMAGE/JPEG')
    );
    const multi = await sut.queryByMediaType('image/JPeg');
    expect(multi).toHaveLength(2);
    expect(multi.some((l) => l.assetId == 'basic113')).toBeTrue();
    expect(multi.some((l) => l.assetId == 'wednesday8')).toBeTrue();
  });

  test('should retrieve pending documents', async function () {
    // setup
    const settingsRepository = new EnvSettingsRepository();
    const sut = new CouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    // zero assets
    expect(await sut.queryNewborn(new Date(1900, 0, 1))).toHaveLength(0);

    // one old asset
    await sut.putAsset(buildBasicAsset('basic113'));
    const one = await sut.queryNewborn(new Date(1900, 0, 1));
    expect(one).toHaveLength(0);

    // multiple assets after 1900
    await sut.putAsset(buildBabyAsset('personN', new Date(1973, 4, 13)));
    await sut.putAsset(buildBabyAsset('personA', new Date(1972, 5, 9)));
    await sut.putAsset(buildBabyAsset('personC', new Date(2005, 9, 14)));
    await sut.putAsset(buildBabyAsset('personJ', new Date(2009, 3, 26)));
    await sut.putAsset(buildBasicAsset('monday6'));
    await sut.putAsset(buildBasicAsset('tuesday7'));
    await sut.putAsset(buildBasicAsset('wednesday8'));
    const nineteen = await sut.queryNewborn(new Date(1900, 0, 1));
    expect(nineteen).toHaveLength(4);
    expect(nineteen.some((l) => l.assetId == 'personN')).toBeTrue();
    expect(nineteen.some((l) => l.assetId == 'personA')).toBeTrue();
    expect(nineteen.some((l) => l.assetId == 'personC')).toBeTrue();
    expect(nineteen.some((l) => l.assetId == 'personJ')).toBeTrue();
    expect(nineteen[0]?.datetime.getFullYear()).toBeGreaterThan(1970);
    expect(nineteen[1]?.datetime.getFullYear()).toBeGreaterThan(1970);
    expect(nineteen[2]?.datetime.getFullYear()).toBeGreaterThan(1970);
    expect(nineteen[3]?.datetime.getFullYear()).toBeGreaterThan(1970);

    // multiple assets after 2000
    await sut.putAsset(buildBabyAsset('personN', new Date(1973, 4, 13)));
    await sut.putAsset(buildBabyAsset('personA', new Date(1972, 5, 9)));
    await sut.putAsset(buildBabyAsset('personC', new Date(2005, 9, 14)));
    await sut.putAsset(buildBabyAsset('personJ', new Date(2009, 3, 26)));
    await sut.putAsset(buildBasicAsset('monday6'));
    await sut.putAsset(buildBasicAsset('tuesday7'));
    await sut.putAsset(buildBasicAsset('wednesday8'));
    const millenium = await sut.queryNewborn(new Date(2000, 0, 1));
    expect(millenium).toHaveLength(2);
    expect(millenium.some((l) => l.assetId == 'personC')).toBeTrue();
    expect(millenium.some((l) => l.assetId == 'personJ')).toBeTrue();
  });

  test('should fetch nothing when database is empty', async function () {
    // setup
    const settingsRepository = new EnvSettingsRepository();
    const sut = new CouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    const [assets, cursor] = await sut.fetchAssets(null, 100);
    expect(assets).toHaveLength(0);
    expect(cursor).toBeDefined();
  });

  test('should fetch assets in a single batch', async function () {
    // setup
    const settingsRepository = new EnvSettingsRepository();
    const sut = new CouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    await sut.putAsset(buildBasicAsset('antonia'));
    await sut.putAsset(buildBasicAsset('christina'));
    await sut.putAsset(buildBasicAsset('joseph'));
    await sut.putAsset(buildBasicAsset('nathan'));

    const [batch1, cursor1] = await sut.fetchAssets(null, 10);
    expect(batch1).toHaveLength(4);
    expect(cursor1).toBeDefined();
    expect(batch1[0]?.key).toEqual('antonia');
    expect(batch1[3]?.key).toEqual('nathan');

    const [batch2, cursor2] = await sut.fetchAssets(cursor1, 10);
    expect(batch2).toHaveLength(0);
  });

  test('should fetch assets in batches', async function () {
    // setup
    const settingsRepository = new EnvSettingsRepository();
    const sut = new CouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    await sut.putAsset(buildBasicAsset('andy'));
    await sut.putAsset(buildBasicAsset('angela'));
    await sut.putAsset(buildBasicAsset('antonia'));
    await sut.putAsset(buildBasicAsset('chachamaru'));
    await sut.putAsset(buildBasicAsset('christina'));
    await sut.putAsset(buildBasicAsset('dickson'));
    await sut.putAsset(buildBasicAsset('eunice'));
    await sut.putAsset(buildBasicAsset('gabriel'));
    await sut.putAsset(buildBasicAsset('gerald'));
    await sut.putAsset(buildBasicAsset('harry'));
    await sut.putAsset(buildBasicAsset('janet'));
    await sut.putAsset(buildBasicAsset('joseph'));
    await sut.putAsset(buildBasicAsset('mittens'));
    await sut.putAsset(buildBasicAsset('nathan'));
    await sut.putAsset(buildBasicAsset('smiley'));
    await sut.putAsset(buildBasicAsset('sonya'));
    await sut.putAsset(buildBasicAsset('stormy'));
    await sut.putAsset(buildBasicAsset('wingtim'));

    const [batch1, cursor1] = await sut.fetchAssets(null, 10);
    // get less than 10 thanks to the _design document taking up space
    expect(batch1).toHaveLength(9);
    expect(cursor1).toBeDefined();
    expect(batch1[0]?.key).toEqual('andy');
    expect(batch1[8]?.key).toEqual('gerald');

    const [batch2, cursor2] = await sut.fetchAssets(cursor1, 10);
    expect(batch2).toHaveLength(9);
    expect(batch2[0]?.key).toEqual('harry');
    expect(batch2[8]?.key).toEqual('wingtim');

    const [batch3, _cursor3] = await sut.fetchAssets(cursor2, 10);
    expect(batch3).toHaveLength(0);
  });

  test('should store assets in bulk', async function () {
    // setup
    const settingsRepository = new EnvSettingsRepository();
    const sut = new CouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    const countBefore = await sut.countAssets();
    expect(countBefore).toBe(0);

    const inputs = [
      buildBasicAsset('andy'),
      buildBasicAsset('angela'),
      buildBasicAsset('antonia'),
      buildBasicAsset('chachamaru'),
      buildBasicAsset('christina'),
      buildBasicAsset('dickson'),
      buildBasicAsset('eunice'),
      buildBasicAsset('gabriel'),
      buildBasicAsset('gerald'),
      buildBasicAsset('harry'),
      buildBasicAsset('janet'),
      buildBasicAsset('joseph'),
      buildBasicAsset('mittens'),
      buildBasicAsset('nathan'),
      buildBasicAsset('smiley'),
      buildBasicAsset('sonya'),
      buildBasicAsset('stormy'),
      buildBasicAsset('wingtim')
    ];
    await sut.storeAssets(inputs);
    const countAfter = await sut.countAssets();
    expect(countAfter).toBe(18);

    // spot check some assets
    const andy = await sut.getAssetById('andy');
    expect(andy?.key).toEqual('andy');
    const chachamaru = await sut.getAssetById('chachamaru');
    expect(chachamaru?.key).toEqual('chachamaru');
    const mittens = await sut.getAssetById('mittens');
    expect(mittens?.key).toEqual('mittens');
    const stormy = await sut.getAssetById('stormy');
    expect(stormy?.key).toEqual('stormy');
    const wingtim = await sut.getAssetById('wingtim');
    expect(wingtim?.key).toEqual('wingtim');
  });
});

/**
 * Construct a simple asset instance.
 *
 * @param key - unique key for the asset.
 * @returns newly generated asset.
 */
function buildBasicAsset(key: string): Asset {
  const checksum = computeKeyHash(key);
  // use a specific date for the date-related tests
  const importDate = new Date(2018, 4, 31, 21, 10, 11); // zero-based month
  const asset = new Asset(key);
  asset.checksum = checksum;
  asset.filename = 'img_1234.jpg';
  asset.byteLength = 1024;
  asset.mediaType = 'image/jpeg';
  asset.tags = ['cat', 'dog'];
  asset.importDate = importDate;
  asset.caption = '#cat and #dog @hawaii';
  asset.location = new Location('hawaii');
  return asset;
}

/**
 * Construct an asset that would appear in the newborn results.
 *
 * @param key - unique key for the asset.
 * @returns newly generated asset.
 */
function buildBabyAsset(key: string, importDate: Date): Asset {
  const checksum = computeKeyHash(key);
  const asset = new Asset(key);
  asset.checksum = checksum;
  asset.filename = 'img_2345.jpg';
  asset.byteLength = 2048;
  asset.mediaType = 'image/jpeg';
  asset.importDate = importDate;
  return asset;
}

/**
 * Generate a sha1-XXX style hash of the given input.
 *
 * @param key - value to be hashed.
 * @returns hash digest with 'sha1-' prefix
 */
function computeKeyHash(key: string): string {
  const hash = crypto.hash('sha1', key);
  return `sha1-${hash}`;
}
