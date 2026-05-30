//
// Copyright (c) 2026 Nathan Fiedler
//
import crypto from 'node:crypto';
import { describe, expect, test } from 'bun:test';
// prepare the test environment as early as possible
import 'tanuki/test/env.ts';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { AssetMetadata } from 'tanuki/server/domain/entities/asset-metadata.ts';
import {
  SyntheticData,
  SyntheticStatus
} from 'tanuki/server/domain/entities/synthetic-data.ts';
import { Location } from 'tanuki/server/domain/entities/location.ts';
import { EnvSettingsRepository } from 'tanuki/server/data/repositories/env-settings-repository.ts';
import { PouchDBRecordRepository } from 'tanuki/server/data/repositories/pouchdb-record-repository.ts';

describe('PouchDBRecordRepository', function () {
  test('should return zero when database is empty', async function () {
    // arrange
    const settingsRepository = new EnvSettingsRepository();
    const sut = new PouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();
    // act
    const count = await sut.countAssets();
    // assert
    expect(count).toEqual(0);
  });

  test('should return null if record cannot be found', async function () {
    // arrange
    const settingsRepository = new EnvSettingsRepository();
    const sut = new PouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();
    // act, assert
    const fetchedById = await sut.getAssetById('foobar');
    expect(fetchedById).toBeNull();
    const fetchedByHash = await sut.getAssetByDigest('cafebabe');
    expect(fetchedByHash).toBeNull();
  });

  test('should store a new document, update, and count', async function () {
    // arrange
    const settingsRepository = new EnvSettingsRepository();
    const sut = new PouchDBRecordRepository({ settingsRepository });
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
    // object returned from PouchDB is an Asset entity and not just a plain
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

  test('should store and delete document', async function () {
    // arrange
    const settingsRepository = new EnvSettingsRepository();
    const sut = new PouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();
    const original = new Asset(
      'MjAxMi8wOC8xNS8wMDB4YWw2czQxYWN0YXY5d2V2Z2VtbXZyYS5qcGc='
    );
    original.setChecksum('sha1-c0ff89017fb951c58aab5e585364a15d359fa2c2');
    // act, assert
    await sut.putAsset(original);
    const fetched = await sut.getAssetById(original.key);
    expect(fetched).toBeDefined();
    await sut.deleteAsset(original.key);
    const gone = await sut.getAssetById(original.key);
    expect(gone).toBeNull();
  });

  test('should retrieve document by checksum', async function () {
    // arrange
    const settingsRepository = new EnvSettingsRepository();
    const sut = new PouchDBRecordRepository({ settingsRepository });
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
    const sut = new PouchDBRecordRepository({ settingsRepository });
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
    const sut = new PouchDBRecordRepository({ settingsRepository });
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
    const sut = new PouchDBRecordRepository({ settingsRepository });
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
    const sut = new PouchDBRecordRepository({ settingsRepository });
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
    const sut = new PouchDBRecordRepository({ settingsRepository });
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
    const sut = new PouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();
    const min_utc = new Date(-8_640_000_000_000_000);
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
    const max_utc = new Date(8_640_000_000_000_000);

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
    const sut = new PouchDBRecordRepository({ settingsRepository });
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
    const sut = new PouchDBRecordRepository({ settingsRepository });
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
    const sut = new PouchDBRecordRepository({ settingsRepository });
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
    const sut = new PouchDBRecordRepository({ settingsRepository });
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
    const sut = new PouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    const [assets, cursor] = await sut.fetchAssets(null, 100);
    expect(assets).toHaveLength(0);
    expect(cursor).toBeDefined();
  });

  test('should fetch assets in a single batch', async function () {
    // setup
    const settingsRepository = new EnvSettingsRepository();
    const sut = new PouchDBRecordRepository({ settingsRepository });
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
    const sut = new PouchDBRecordRepository({ settingsRepository });
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
    // get less than 10 thanks to the _design document(s) taking up space
    expect(batch1).toHaveLength(8);
    expect(cursor1).toBeDefined();
    expect(batch1[0]?.key).toEqual('andy');
    expect(batch1[7]?.key).toEqual('gabriel');

    const [batch2, cursor2] = await sut.fetchAssets(cursor1, 10);
    expect(batch2).toHaveLength(10);
    expect(batch2[0]?.key).toEqual('gerald');
    expect(batch2[9]?.key).toEqual('wingtim');

    const [batch3, _cursor3] = await sut.fetchAssets(cursor2, 10);
    expect(batch3).toHaveLength(0);
  });

  test('should store assets in bulk', async function () {
    // setup
    const settingsRepository = new EnvSettingsRepository();
    const sut = new PouchDBRecordRepository({ settingsRepository });
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

  test('should round-trip AssetMetadata via putAsset and getAssetById', async function () {
    const settingsRepository = new EnvSettingsRepository();
    const sut = new PouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    const asset = buildBasicAsset('meta-roundtrip');
    const meta = new AssetMetadata();
    meta.cameraMake = 'Canon';
    meta.cameraModel = 'EOS 5D Mark IV';
    meta.fNumber = 2.8;
    meta.iso = 400;
    meta.originalDateOffset = '+09:00';
    meta.gpsLatitude = 35.6895;
    meta.gpsLongitude = 139.6917;
    meta.displayWidth = 6720;
    meta.displayHeight = 4480;
    meta.raw = { Make: { description: 'Canon' } };
    asset.metadata = meta;
    await sut.putAsset(asset);

    const fetched = await sut.getAssetById('meta-roundtrip');
    expect(fetched?.metadata).not.toBeNull();
    expect(fetched?.metadata?.cameraMake).toEqual('Canon');
    expect(fetched?.metadata?.cameraModel).toEqual('EOS 5D Mark IV');
    expect(fetched?.metadata?.fNumber).toBeCloseTo(2.8);
    expect(fetched?.metadata?.iso).toEqual(400);
    expect(fetched?.metadata?.originalDateOffset).toEqual('+09:00');
    expect(fetched?.metadata?.gpsLatitude).toBeCloseTo(35.6895);
    expect(fetched?.metadata?.gpsLongitude).toBeCloseTo(139.6917);
    expect(fetched?.metadata?.displayWidth).toEqual(6720);
    expect(fetched?.metadata?.displayHeight).toEqual(4480);
    expect(fetched?.metadata?.raw).toEqual({ Make: { description: 'Canon' } });

    // updating with metadata=null should drop the sub-document, leaving a
    // byteLength-only metadata on read.
    asset.metadata = null;
    await sut.putAsset(asset);
    const cleared = await sut.getAssetById('meta-roundtrip');
    expect(cleared?.metadata?.hasValues()).toBe(false);
    expect(cleared?.metadata?.byteLength).toEqual(asset.byteLength);
  });

  test('fetchMetadata should return a map keyed by asset id with missing entries as null', async function () {
    const settingsRepository = new EnvSettingsRepository();
    const sut = new PouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    const withMeta = buildBasicAsset('with-meta');
    const meta = new AssetMetadata();
    meta.cameraMake = 'Sony';
    meta.fNumber = 1.8;
    withMeta.metadata = meta;
    await sut.putAsset(withMeta);

    const withoutMeta = buildBasicAsset('without-meta');
    await sut.putAsset(withoutMeta);

    const result = await sut.fetchMetadata([
      'with-meta',
      'without-meta',
      'does-not-exist'
    ]);
    expect(result.size).toEqual(3);
    expect(result.get('with-meta')?.cameraMake).toEqual('Sony');
    expect(result.get('with-meta')?.fNumber).toBeCloseTo(1.8);
    // Without extracted metadata, the repo still returns a metadata object so
    // callers can read `byteLength` (sourced from the parent asset doc).
    expect(result.get('without-meta')?.hasValues()).toBe(false);
    expect(result.get('without-meta')?.byteLength).toEqual(withoutMeta.byteLength);
    expect(result.get('does-not-exist')).toBeNull();

    const empty = await sut.fetchMetadata([]);
    expect(empty.size).toEqual(0);
  });

  test('synthetic data should default to PENDING with no sub-document', async function () {
    const settingsRepository = new EnvSettingsRepository();
    const sut = new PouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    const asset = buildBasicAsset('synth-default');
    await sut.putAsset(asset);

    const fetched = await sut.getAssetById('synth-default');
    expect(fetched?.synthetic).toBeNull();
    expect(fetched?.syntheticStatus).toEqual(SyntheticStatus.PENDING);

    const dataMap = await sut.fetchSynthetic(['synth-default']);
    const statusMap = await sut.fetchSyntheticStatus(['synth-default']);
    expect(dataMap.get('synth-default')).toBeNull();
    expect(statusMap.get('synth-default')).toEqual(SyntheticStatus.PENDING);
  });

  test('putAsset should round-trip synthetic data on the Asset entity', async function () {
    const settingsRepository = new EnvSettingsRepository();
    const sut = new PouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    const asset = buildBasicAsset('synth-put-roundtrip');
    const data = new SyntheticData();
    data.labels = ['cat', 'sofa'];
    data.primaryLabel = 'cat';
    asset.synthetic = data;
    asset.syntheticStatus = SyntheticStatus.READY;
    await sut.putAsset(asset);

    const fetched = await sut.getAssetById('synth-put-roundtrip');
    expect(fetched?.synthetic?.labels).toEqual(['cat', 'sofa']);
    expect(fetched?.synthetic?.primaryLabel).toEqual('cat');
    expect(fetched?.syntheticStatus).toEqual(SyntheticStatus.READY);
  });

  test('setSynthetic should round-trip labels and status', async function () {
    const settingsRepository = new EnvSettingsRepository();
    const sut = new PouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    const asset = buildBasicAsset('synth-roundtrip');
    await sut.putAsset(asset);

    const data = new SyntheticData();
    data.labels = ['beach', 'palm tree', 'sunset'];
    data.primaryLabel = 'beach';
    await sut.setSynthetic('synth-roundtrip', data, SyntheticStatus.READY);

    const fetched = await sut.getAssetById('synth-roundtrip');
    expect(fetched?.synthetic?.labels).toEqual(['beach', 'palm tree', 'sunset']);
    expect(fetched?.synthetic?.primaryLabel).toEqual('beach');
    expect(fetched?.syntheticStatus).toEqual(SyntheticStatus.READY);
  });

  test('setSynthetic with FAILED status persists status with no data', async function () {
    const settingsRepository = new EnvSettingsRepository();
    const sut = new PouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    const asset = buildBasicAsset('synth-failed');
    await sut.putAsset(asset);

    await sut.setSynthetic('synth-failed', null, SyntheticStatus.FAILED);
    const fetched = await sut.getAssetById('synth-failed');
    expect(fetched?.synthetic).toBeNull();
    expect(fetched?.syntheticStatus).toEqual(SyntheticStatus.FAILED);
  });

  test('fetchSynthetic batch returns a map with missing entries as null', async function () {
    const settingsRepository = new EnvSettingsRepository();
    const sut = new PouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    const withSynth = buildBasicAsset('with-synth');
    await sut.putAsset(withSynth);
    const data = new SyntheticData();
    data.labels = ['dog'];
    data.primaryLabel = 'dog';
    await sut.setSynthetic('with-synth', data, SyntheticStatus.READY);

    const withoutSynth = buildBasicAsset('without-synth');
    await sut.putAsset(withoutSynth);

    const dataMap = await sut.fetchSynthetic([
      'with-synth',
      'without-synth',
      'does-not-exist'
    ]);
    expect(dataMap.size).toEqual(3);
    expect(dataMap.get('with-synth')?.primaryLabel).toEqual('dog');
    expect(dataMap.get('without-synth')).toBeNull();
    expect(dataMap.get('does-not-exist')).toBeNull();

    const statusMap = await sut.fetchSyntheticStatus([
      'with-synth',
      'without-synth',
      'does-not-exist'
    ]);
    expect(statusMap.get('with-synth')).toEqual(SyntheticStatus.READY);
    expect(statusMap.get('without-synth')).toEqual(SyntheticStatus.PENDING);
    expect(statusMap.get('does-not-exist')).toEqual(SyntheticStatus.PENDING);
  });

  test('allPrimaryLabels groups by primary_label and counts assets', async function () {
    const settingsRepository = new EnvSettingsRepository();
    const sut = new PouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    const a = buildBasicAsset('label-a');
    const b = buildBasicAsset('label-b');
    const c = buildBasicAsset('label-c');
    const d = buildBasicAsset('label-d');
    for (const asset of [a, b, c, d]) await sut.putAsset(asset);
    const beach = new SyntheticData();
    beach.labels = ['beach'];
    beach.primaryLabel = 'beach';
    const cat = new SyntheticData();
    cat.labels = ['cat'];
    cat.primaryLabel = 'cat';
    await sut.setSynthetic('label-a', beach, SyntheticStatus.READY);
    await sut.setSynthetic('label-b', beach, SyntheticStatus.READY);
    await sut.setSynthetic('label-c', cat, SyntheticStatus.READY);

    const all = await sut.allPrimaryLabels();
    all.sort((x, y) => x.label.localeCompare(y.label));
    expect(all).toHaveLength(2);
    expect(all[0]?.label).toEqual('beach');
    expect(all[0]?.count).toEqual(2);
    expect(all[1]?.label).toEqual('cat');
    expect(all[1]?.count).toEqual(1);
  });

  test('queryByLabel returns assets whose primaryLabel matches (case-insensitive)', async function () {
    const settingsRepository = new EnvSettingsRepository();
    const sut = new PouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    const a = buildBasicAsset('q-a');
    const b = buildBasicAsset('q-b');
    const c = buildBasicAsset('q-c');
    for (const asset of [a, b, c]) await sut.putAsset(asset);
    const beach = new SyntheticData();
    beach.labels = ['beach'];
    beach.primaryLabel = 'Beach';
    const cat = new SyntheticData();
    cat.labels = ['cat'];
    cat.primaryLabel = 'cat';
    await sut.setSynthetic('q-a', beach, SyntheticStatus.READY);
    await sut.setSynthetic('q-b', beach, SyntheticStatus.READY);
    await sut.setSynthetic('q-c', cat, SyntheticStatus.READY);

    const matches = await sut.queryByLabel('beach');
    const ids = matches.map((r) => r.assetId).sort();
    expect(ids).toEqual(['q-a', 'q-b']);

    const cats = await sut.queryByLabel('cat');
    expect(cats.map((r) => r.assetId)).toEqual(['q-c']);

    const none = await sut.queryByLabel('nothing-here');
    expect(none).toEqual([]);
  });

  test('latestAssetByLabel returns the most-recent asset with its case-preserved label', async function () {
    const settingsRepository = new EnvSettingsRepository();
    const sut = new PouchDBRecordRepository({ settingsRepository });
    await sut.destroyAndCreate();

    const older = buildBasicAsset('latest-older');
    older.setUserDate(new Date(2020, 0, 1));
    const newer = buildBasicAsset('latest-newer');
    newer.setUserDate(new Date(2025, 5, 1));
    const other = buildBasicAsset('latest-other');
    await sut.putAsset(older);
    await sut.putAsset(newer);
    await sut.putAsset(other);
    const beach = new SyntheticData();
    beach.labels = ['Beach'];
    beach.primaryLabel = 'Beach';
    const cat = new SyntheticData();
    cat.labels = ['cat'];
    cat.primaryLabel = 'cat';
    await sut.setSynthetic('latest-older', beach, SyntheticStatus.READY);
    await sut.setSynthetic('latest-newer', beach, SyntheticStatus.READY);
    await sut.setSynthetic('latest-other', cat, SyntheticStatus.READY);

    const result = await sut.latestAssetByLabel('beach');
    expect(result).not.toBeNull();
    expect(result!.assetId).toEqual('latest-newer');
    expect(result!.primaryLabel).toEqual('Beach');
    expect(await sut.latestAssetByLabel('nothing')).toBeNull();
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
