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
    const doc = new Asset('rabbit');
    doc.checksum = 'sha1-cafed00d';
    // act
    await sut.putAsset(doc);
    // assert
    let asset = await sut.getAssetById(doc.key);
    expect(asset).toBeDefined();
    expect(asset!.key).toEqual('rabbit');
    expect(asset!.checksum).toEqual(doc.checksum);
    expect(asset!.importDate).toEqual(doc.importDate);
    let count = await sut.countAssets();
    expect(count).toEqual(1);

    // update the document
    const newdoc = Object.assign({}, doc, { tags: ['rabbit', 'bunny'] });
    await sut.putAsset(newdoc);
    asset = await sut.getAssetById(doc.key);
    expect(asset).toBeDefined();
    expect(asset!.key).toEqual('rabbit');
    expect(asset!.checksum).toEqual(doc.checksum);
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
    let asset = await sut.getAssetByDigest(doc.checksum);
    expect(asset).toBeDefined();
    expect(asset!.key).toEqual('eagle');
    expect(asset!.checksum).toEqual(doc.checksum);
    expect(asset!.importDate).toEqual(doc.importDate);
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
    await sut.putAsset(buildBasicAsset('monday6').setFilename('img_2345.jpg').setTags(['bird', 'dog']));
    await sut.putAsset(buildBasicAsset('tuesday7').setFilename('img_3456.jpg').setUserDate(
      new Date(2004, 4, 31, 21, 10, 11)
    ).setTags(['CAT', 'mouse']));
    await sut.putAsset(buildBasicAsset('wednesday8').setFilename('img_4567.jpg').setUserDate(
      new Date(2007, 4, 31, 21, 10, 11)
    ).setTags(['Cat', 'lizard', 'chicken']));
    await sut.putAsset(buildBasicAsset('thursday9').setFilename('img_5678.jpg').setTags(['bird', 'dog']));
    await sut.putAsset(buildBasicAsset('friday10').setFilename('img_6789.jpg').setTags(['mouse', 'house']));
    const multi = await sut.queryByTags(['cAt']);
    expect(multi).toHaveLength(3);
    expect(multi.every((l) => l.mediaType == 'image/jpeg')).toBeTrue();
    expect(multi.every((l) => l.location!.label == 'hawaii')).toBeTrue();
    expect(multi.some((l) => l.assetId == 'basic123' && l.filename == 'img_1234.jpg' && l.datetime.getFullYear() == 2018)).toBeTrue();
    expect(multi.some((l) => l.assetId == 'tuesday7' && l.filename == 'img_3456.jpg' && l.datetime.getFullYear() == 2004)).toBeTrue();
    expect(multi.some((l) => l.assetId == 'wednesday8' && l.filename == 'img_4567.jpg' && l.datetime.getFullYear() == 2007)).toBeTrue();
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
    await sut.putAsset(buildBasicAsset('year_1940').setUserDate(year_1940).setImportDate(year_2011));
    await sut.putAsset(buildBasicAsset('year_1996').setOriginalDate(year_1996));
    await sut.putAsset(buildBasicAsset('year_2003').setOriginalDate(year_2003));
    await sut.putAsset(buildBasicAsset('year_2008').setOriginalDate(year_2008));
    await sut.putAsset(buildBasicAsset('year_2011').setUserDate(year_2011).setImportDate(year_2019));
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
    expect(result_after_2008.some((l) => l.assetId == 'future_date')).toBeTrue();

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
    await sut.putAsset(buildBasicAsset('monday6').setFilename('img_2345.jpg').
      setLocation(Location.parse('Paris, France')));
    await sut.putAsset(buildBasicAsset('monday8').setFilename('img_6543.jpg').
      setLocation(Location.parse('Nice, France')));
    await sut.putAsset(buildBasicAsset('tuesday7').setFilename('img_3456.jpg').
      setLocation(new Location('london')));
    await sut.putAsset(buildBasicAsset('wednesday8').setFilename('img_4567.jpg').
      setLocation(new Location('seoul')));
    await sut.putAsset(buildBasicAsset('thursday9').setFilename('img_5678.jpg').
      setLocation(Location.fromParts('', 'oahu', 'hawaii')));
    await sut.putAsset(buildBasicAsset('friday10').setFilename('img_6789.jpg').
      setLocation(new Location('paris')));
    await sut.putAsset(buildBasicAsset('friday11').setFilename('img_6879.jpg').
      setLocation(Location.fromParts('city center', 'portland', 'OR')));

    // searching with a single location
    const single = await sut.queryByLocations(['hawaii']);
    expect(single).toHaveLength(2);
    expect(single.some((l) => l.assetId == 'basic113')).toBeTrue();
    expect(single.some((l) => l.assetId == 'thursday9')).toBeTrue();

    // searching with multiple locations
    const multiple = await sut.queryByLocations(['hawaii', 'oahu']);
    expect(multiple).toHaveLength(1);
    expect(multiple[0]?.assetId).toEqual('thursday9');

    // searching location term split from commas
    const france = await sut.queryByLocations(['france']);
    expect(france).toHaveLength(2);
    expect(france.some((l) => l.assetId == 'monday6')).toBeTrue();
    expect(france.some((l) => l.assetId == 'monday8')).toBeTrue();

    // searching location term from region field
    const oregon = await sut.queryByLocations(['or']);
    expect(oregon).toHaveLength(1);
    expect(oregon[0]?.assetId).toEqual('friday11');
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
    await sut.putAsset(buildBasicAsset('monday6').setFilename('img_2345.jpg').setMediaType('image/png'));
    await sut.putAsset(buildBasicAsset('tuesday7').setFilename('img_3456.jpg').setMediaType('video/mpeg'));
    await sut.putAsset(buildBasicAsset('wednesday8').setFilename('img_4567.jpg').setMediaType('IMAGE/JPEG'));
    const multi = await sut.queryByMediaType('image/JPeg');
    expect(multi).toHaveLength(2);
    expect(multi.some((l) => l.assetId == 'basic113')).toBeTrue();
    expect(multi.some((l) => l.assetId == 'wednesday8')).toBeTrue();
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
 * Generate a sha1-XXX style hash of the given input.
 * 
 * @param key - value to be hashed.
 * @returns hash digest with 'sha1-' prefix
 */
function computeKeyHash(key: string): string {
  const hash = crypto.hash('sha1', key);
  return `sha1-${hash}`;
}
