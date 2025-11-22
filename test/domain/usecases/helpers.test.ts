//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, test } from "bun:test";
import * as helpers from '../../../server/domain/usecases/usecases';

describe('Use case helpers', function () {
  test('should compute checksum of a file', async function () {
    // arrange
    const filepath = 'test/fixtures/dcp_1069.jpg';
    const expected = 'sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07';
    // act
    const actual = await helpers.checksumFile(filepath);
    // assert
    expect(actual).toEqual(expected);
  });

  test('should generate an asset identifier', async function () {
    // arrange
    const datetime = new Date(2018, 4, 31, 21, 10, 11); // months are 0-based
    const mimetype = 'image/jpeg';
    // act
    const actual = helpers.newAssetId(datetime, mimetype);
    // assert
    const buf = Buffer.from(actual, 'base64');
    const decoded = buf.toString('utf8');
    expect(decoded.startsWith('2018/05/31/2100/')).toBeTrue();
    expect(decoded.endsWith('.jpg')).toBeTrue();
    expect(decoded).toHaveLength(46);
  });

  // TODO: newAssetId with incorrect file extension
  //     // test with an image/jpeg asset with an incorrect extension
  //     let filename = "fighting_kittens.foo";
  //     let actual = new_asset_id(import_date, Path::new(filename), &mt);
  //     let decoded = general_purpose::STANDARD.decode(&actual).unwrap();
  //     let as_string = std::str::from_utf8(&decoded).unwrap();
  //     assert!(as_string.ends_with(".foo.jpeg"));

  // TODO: newAssetId with an image/jpeg asset with _no_ extension
  //     let filename = "fighting_kittens";
  //     let actual = new_asset_id(import_date, Path::new(filename), &mt);
  //     let decoded = general_purpose::STANDARD.decode(&actual).unwrap();
  //     let as_string = std::str::from_utf8(&decoded).unwrap();
  //     assert!(as_string.ends_with(".jpeg"));

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

  //     // MP4-encoded quicktime/mpeg video file
  //     let filename = "./test/fixtures/100_1206.MOV";
  //     let mt: mime::Mime = "video/mp4".parse().unwrap();
  //     let filepath = Path::new(filename);
  //     let actual = get_original_date(&mt, filepath);
  //     // note that get_original_date() is sensitive to the mp4 crate's ability
  //     // to parse the file successfully, resulting in misleading errors
  //     assert!(actual.is_ok());
  //     let date = actual.unwrap();
  //     assert_eq!(date.year(), 2007);
  //     assert_eq!(date.month(), 9);
  //     assert_eq!(date.day(), 14);

  //     // MP4 file with out-of-order tracks
  //     let filename = "./test/fixtures/ooo_tracks.mp4";
  //     let mt: mime::Mime = "video/mp4".parse().unwrap();
  //     let filepath = Path::new(filename);
  //     let actual = get_original_date(&mt, filepath);
  //     assert!(actual.is_ok());
  //     let date = actual.unwrap();
  //     assert_eq!(date.year(), 2016);
  //     assert_eq!(date.month(), 9);
  //     assert_eq!(date.day(), 5);

  //     // RIFF-encoded AVI video file
  //     let filename = "./test/fixtures/MVI_0727.AVI";
  //     let mt: mime::Mime = "video/x-msvideo".parse().unwrap();
  //     let filepath = Path::new(filename);
  //     let actual = get_original_date(&mt, filepath);
  //     assert!(actual.is_ok());
  //     let date = actual.unwrap();
  //     assert_eq!(date.year(), 2009);
  //     assert_eq!(date.month(), 1);
  //     assert_eq!(date.day(), 19);

  //     // not an actual video, despite the media type
  //     let filename = "./test/fixtures/lorem-ipsum.txt";
  //     let filepath = Path::new(filename);
  //     let actual = get_original_date(&mt, filepath);
  //     assert!(actual.is_err());
});
