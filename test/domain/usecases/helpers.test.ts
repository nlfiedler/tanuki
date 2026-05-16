//
// Copyright (c) 2025 Nathan Fiedler
//
import path from 'node:path';
import { describe, expect, test } from 'bun:test';
import { Geocoded, Location } from 'tanuki/server/domain/entities/location.ts';
import * as helpers from 'tanuki/server/domain/usecases/helpers.ts';

describe('File checksum helper', function () {
  test('should compute checksum of a file', async function () {
    // arrange
    const filepath = 'test/fixtures/dcp_1069.jpg';
    const expected =
      'sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07';
    // act
    const actual = await helpers.checksumFile(filepath);
    // assert
    expect(actual).toEqual(expected);
  });
});

describe('Generate identifier helper', function () {
  test('should generate an asset identifier', async function () {
    // arrange
    const datetime = new Date(2018, 4, 31, 21, 10, 11); // months are 0-based
    const mimetype = 'image/jpeg';
    // act
    const actual = helpers.newAssetId(datetime, mimetype);
    // assert
    const buf = Buffer.from(actual, 'base64url');
    const decoded = buf.toString('utf8');
    const prefix = path.normalize('2018/05/31/2100/');
    expect(decoded.startsWith(prefix)).toBeTrue();
    expect(decoded.endsWith('.jpg')).toBeTrue();
    expect(decoded).toHaveLength(46);
  });
});

describe('EXIF original date-time', function () {
  test('should read EXIF original date/time from JPEG', async function () {
    // arrange
    const mimetype = 'image/jpeg';
    const filepath = 'test/fixtures/dcp_1069.jpg';
    // act
    const raw = await helpers.getOriginalDate(mimetype, filepath);
    const actual = new Date(raw || 0);
    // assert
    expect(actual.getFullYear()).toEqual(2003);
    expect(actual.getMonth()).toEqual(8); // zero-based
    expect(actual.getDay()).toEqual(3);
  });

  test('should return null when reading EXIF from JPEG w/o a date', async function () {
    // arrange
    const mimetype = 'image/jpeg';
    const filepath = 'test/fixtures/fighting_kittens.jpg';
    // act
    const actual = await helpers.getOriginalDate(mimetype, filepath);
    // assert
    expect(actual).toBeNull();
  });

  test('should return null when reading image w/o EXIF data', async function () {
    // arrange
    const mimetype = 'image/jpeg';
    const filepath = 'test/fixtures/animal-cat-cute-126407.jpg';
    // act
    const actual = await helpers.getOriginalDate(mimetype, filepath);
    // assert
    expect(actual).toBeNull();
  });

  test('should return null when reading file that is not an image', async function () {
    // arrange
    const mimetype = 'image/jpeg';
    const filepath = 'test/fixtures/lorem-ipsum.txt';
    // act
    const actual = await helpers.getOriginalDate(mimetype, filepath);
    // assert
    expect(actual).toBeNull();
  });

  test('should read creation_time from MOV', async function () {
    // arrange
    const mimetype = 'video/quicktime';
    const filepath = 'test/fixtures/100_1206.MOV';
    // act
    const raw = await helpers.getOriginalDate(mimetype, filepath);
    const actual = new Date(raw || 0);
    // assert
    expect(actual.getUTCFullYear()).toEqual(2007);
    expect(actual.getUTCMonth()).toEqual(8); // zero-based: September
    expect(actual.getUTCDate()).toEqual(14);
  });

  test('should read creation_time from MP4 with out-of-order tracks', async function () {
    // arrange
    const mimetype = 'video/mp4';
    const filepath = 'test/fixtures/ooo_tracks.mp4';
    // act
    const raw = await helpers.getOriginalDate(mimetype, filepath);
    const actual = new Date(raw || 0);
    // assert
    expect(actual.getUTCFullYear()).toEqual(2016);
    expect(actual.getUTCMonth()).toEqual(8); // zero-based: September
    expect(actual.getUTCDate()).toEqual(5);
  });

  test('should return null for non-video file with video mimetype', async function () {
    // arrange
    const mimetype = 'video/mp4';
    const filepath = 'test/fixtures/lorem-ipsum.txt';
    // act
    const actual = await helpers.getOriginalDate(mimetype, filepath);
    // assert
    expect(actual).toBeNull();
  });
});

describe('EXIF GPS coordinates', function () {
  test('should read GPS coords', async function () {
    // arrange
    const mimetype = 'image/jpeg';
    const filepath = 'test/fixtures/IMG_0385.JPG';
    // act
    const actual = await helpers.getCoordinates(mimetype, filepath);
    // assert
    expect(actual?.latitudeRef).toEqual('N');
    expect(actual?.latitude[0]).toEqual(37);
    expect(actual?.latitude[1]).toEqual(42);
    expect(actual?.latitude[2]).toEqual(31.93);
    expect(actual?.longitudeRef).toEqual('W');
    expect(actual?.longitude[0]).toEqual(122);
    expect(actual?.longitude[1]).toEqual(3);
    expect(actual?.longitude[2]).toEqual(47.72);
  });
});

describe('extractImageInfo', function () {
  test('returns metadata, original date, and coords for an image with EXIF', async function () {
    const info = await helpers.extractImageInfo('test/fixtures/IMG_0385.JPG');
    expect(info).not.toBeNull();
    expect(info!.coordinates).not.toBeNull();
    expect(info!.metadata.gpsLatitude).toBeGreaterThan(37);
    expect(info!.metadata.gpsLongitude).toBeLessThan(0);
    expect(info!.metadata.hasValues()).toBeTrue();
  });

  test('returns null for a non-image file', async function () {
    const info = await helpers.extractImageInfo('test/fixtures/lorem-ipsum.txt');
    expect(info).toBeNull();
  });

  test('returns displayWidth/displayHeight for f1t.jpg', async function () {
    const info = await helpers.extractImageInfo('test/fixtures/f1t.jpg');
    expect(info).not.toBeNull();
    expect(info!.metadata.displayWidth).toEqual(48);
    expect(info!.metadata.displayHeight).toEqual(80);
  });

  test('falls back to sharp dimensions when ExifReader fails', async function () {
    // SVG is an image format sharp can read but ExifReader rejects, forcing
    // the imageInfoFromDimensions fallback path.
    const info = await helpers.extractImageInfo('test/fixtures/red-rect.svg');
    expect(info).not.toBeNull();
    expect(info!.metadata.displayWidth).toEqual(80);
    expect(info!.metadata.displayHeight).toEqual(46);
  });
});

describe('parseVideoMetadata', function () {
  test('extracts duration, codec, and rotation-adjusted dimensions', function () {
    // synthesized ffprobe output for a portrait video with Display Matrix
    const probe = {
      format: {
        duration: '10.500000',
        tags: { creation_time: '2024-01-15T10:30:00.000000Z' }
      },
      streams: [
        {
          codec_type: 'video',
          codec_name: 'h264',
          width: 1920,
          height: 1080,
          r_frame_rate: '30000/1001',
          side_data_list: [{ rotation: -90 }]
        }
      ]
    };
    const { metadata, originalDate } = helpers.parseVideoMetadata(probe);
    expect(metadata.duration).toBeCloseTo(10.5);
    expect(metadata.videoCodec).toEqual('h264');
    expect(metadata.frameRate).toBeCloseTo(29.97, 2);
    // sideways rotation swaps width/height
    expect(metadata.displayWidth).toEqual(1080);
    expect(metadata.displayHeight).toEqual(1920);
    expect(originalDate).toEqual(Date.UTC(2024, 0, 15, 10, 30, 0));
  });

  test('returns null fields when probe has no video stream', function () {
    const probe = { format: { duration: '1.0' }, streams: [] };
    const { metadata } = helpers.parseVideoMetadata(probe);
    expect(metadata.videoCodec).toBeNull();
    expect(metadata.duration).toBeCloseTo(1);
  });

  test('honors stream.tags.rotate fallback', function () {
    const probe = {
      streams: [
        {
          codec_type: 'video',
          codec_name: 'h264',
          width: 1920,
          height: 1080,
          tags: { rotate: '90' }
        }
      ]
    };
    const { metadata } = helpers.parseVideoMetadata(probe);
    expect(metadata.displayWidth).toEqual(1080);
    expect(metadata.displayHeight).toEqual(1920);
  });
});

describe('Merge locations helper', function () {
  test('should merge various location objects', async function () {
    // both are none, result is none
    let asset: Location | null = null;
    let input: Location | null = null;
    let result = helpers.mergeLocations(asset, input);
    expect(result).toBeNull();

    // asset is some, input is none, result is asset
    asset = Location.fromParts('beach', 'Monterey', 'California');
    input = null;
    result = helpers.mergeLocations(asset, input);
    expect(result).toEqual(asset);
    expect(result?.hasValues()).toBeTrue();

    // asset is none, input is returned
    asset = null;
    input = Location.parse('Seattle, WA');
    result = helpers.mergeLocations(asset, input);
    expect(result).toEqual(input);
    expect(result?.hasValues()).toBeTrue();

    // merge input city/region with asset label
    asset = new Location('Chihuly');
    input = Location.parse('Seattle, WA');
    result = helpers.mergeLocations(asset, input);
    expect(result).toEqual(Location.parse('Chihuly; Seattle, WA'));
    expect(result?.hasValues()).toBeTrue();

    // merge input label with asset city/region
    asset = Location.parse('Seattle, WA');
    input = new Location('Chihuly');
    result = helpers.mergeLocations(asset, input);
    expect(result).toEqual(Location.parse('Chihuly; Seattle, WA'));
    expect(result?.hasValues()).toBeTrue();

    // clear asset label if input label is empty string or null
    asset = new Location('Chihuly');
    input = Location.fromRaw('', 'Seattle', 'WA');
    result = helpers.mergeLocations(asset, input);
    expect(result).toEqual(Location.parse('Seattle, WA'));
    expect(result?.hasValues()).toBeTrue();

    // clear asset city if input city is empty string
    asset = Location.parse('museum; Seattle, WA');
    input = Location.fromRaw(null, '', null);
    result = helpers.mergeLocations(asset, input);
    expect(result).toEqual(Location.fromRaw('museum', null, 'WA'));
    expect(result?.hasValues()).toBeTrue();

    // clear asset region if input region is empty string
    asset = Location.parse('museum; Seattle, WA');
    input = Location.fromRaw(null, null, '');
    result = helpers.mergeLocations(asset, input);
    expect(result).toEqual(Location.fromRaw('museum', 'Seattle', null));
    expect(result?.hasValues()).toBeTrue();

    // input with everything replaces everything in asset
    asset = Location.parse('Chihuly; Seattle, WA');
    input = Location.parse('Classical Garden; Portland, Oregon');
    result = helpers.mergeLocations(asset, input);
    expect(result).toEqual(input);
    expect(result?.hasValues()).toBeTrue();
  });
});

describe('Geocode location converter', function () {
  test('should convert various geocoded locations', async function () {
    // city is none but region and country are defined
    let input = new Geocoded(null, 'New Territories', 'Hong Kong');
    let actual = helpers.locationFromGeocoded(input);
    expect(actual.city).toEqual('New Territories');
    expect(actual.region).toEqual('Hong Kong');

    // country is not needed
    input = new Geocoded('Portland', 'Oregon', 'United States');
    actual = helpers.locationFromGeocoded(input);
    expect(actual.city).toEqual('Portland');
    expect(actual.region).toEqual('Oregon');

    // city equals region
    input = new Geocoded('Nara', 'Nara', 'Japan');
    actual = helpers.locationFromGeocoded(input);
    expect(actual.city).toEqual('Nara');
    expect(actual.region).toEqual('Japan');

    // region has city as prefix
    input = new Geocoded('Jerusalem', 'Jerusalem District', 'Israel');
    actual = helpers.locationFromGeocoded(input);
    expect(actual.city).toEqual('Jerusalem');
    expect(actual.region).toEqual('Israel');

    // region has city as suffix
    input = new Geocoded('São Paulo', 'State of São Paulo', 'Brazil');
    actual = helpers.locationFromGeocoded(input);
    expect(actual.city).toEqual('São Paulo');
    expect(actual.region).toEqual('Brazil');

    // all blank fields
    input = new Geocoded(null, null, null);
    actual = helpers.locationFromGeocoded(input);
    expect(actual.city).toBeNull();
    expect(actual.region).toBeNull();

    // no city or region
    input = new Geocoded('Portland', null, null);
    actual = helpers.locationFromGeocoded(input);
    expect(actual.city).toEqual('Portland');
    expect(actual.region).toBeNull();
  });
});

describe('parseCaption helper', function () {
  test('should return nothing if caption is plain text', async function () {
    const results = helpers.parseCaption(
      'this is plain text without any markup'
    );
    expect(results.tags).toBeEmpty();
    expect(results.location).toBeNull();
  });

  test('should return simple location label', async function () {
    const results = helpers.parseCaption(
      'on the beach @hawaii, fun in the sun'
    );
    expect(results.tags).toBeEmpty();
    expect(results.location?.label).toEqual('hawaii');
  });

  test('should return complex location value', async function () {
    const results = helpers.parseCaption(
      'fun in the sun @"beach; Oahu, Hawaii"'
    );
    expect(results.tags).toBeEmpty();
    expect(results.location?.label).toEqual('beach');
    expect(results.location?.city).toEqual('Oahu');
    expect(results.location?.region).toEqual('Hawaii');
  });

  test('should return simple location and multiple tags', async function () {
    const results = helpers.parseCaption('#fun #sun #beach @hawaii');
    expect(results.tags).toHaveLength(3);
    expect(results.tags[0]).toEqual('fun');
    expect(results.tags[1]).toEqual('sun');
    expect(results.tags[2]).toEqual('beach');
    expect(results.location?.label).toEqual('hawaii');
  });

  test('should find tags wrapped in parentheses', async function () {
    const results = helpers.parseCaption('(#nathan #oma #opa)');
    expect(results.tags).toHaveLength(3);
    expect(results.tags[0]).toEqual('nathan');
    expect(results.tags[1]).toEqual('oma');
    expect(results.tags[2]).toEqual('opa');
    expect(results.location).toBeNull();
  });

  test('should find tags surrounded by separators', async function () {
    const results = helpers.parseCaption(
      '#cat. #dog, #bird #mouse; #house(#car)'
    );
    expect(results.tags).toHaveLength(6);
    expect(results.tags[0]).toEqual('cat');
    expect(results.tags[1]).toEqual('dog');
    expect(results.tags[2]).toEqual('bird');
    expect(results.tags[3]).toEqual('mouse');
    expect(results.tags[4]).toEqual('house');
    expect(results.tags[5]).toEqual('car');
    expect(results.location).toBeNull();
  });
});
