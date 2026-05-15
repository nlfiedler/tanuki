//
// Copyright (c) 2025 Nathan Fiedler
//
import fs from 'node:fs/promises';
import path from 'node:path';
import { afterEach, describe, expect, test } from 'bun:test';
import { temporaryDirectory } from 'tempy';
import { clearFetchMocks, mockFetch } from '@aryzing/bun-mock-fetch';
// prepare the test environment as early as possible
import 'tanuki/test/env.ts';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { EnvSettingsRepository } from 'tanuki/server/data/repositories/env-settings-repository.ts';
import { NamazuBlobRepository } from 'tanuki/server/data/repositories/namazu-blob-repository.ts';

describe('NamazuBlobRepository', function () {
  afterEach(() => {
    clearFetchMocks();
  });

  test('should produce a URL for the full asset', async function () {
    // arrange
    const assetId =
      'MjAxOC8wNS8zMS8yMTAwLzAxYng1enprYmthY3Rhdjl3ZXZnZW1tdnJ6LmpwZw==';
    const expected = 'http://example.com/assets/' + assetId;
    const settingsRepository = new EnvSettingsRepository();
    settingsRepository.set('NAMAZU_URL', 'http://example.com');
    const sut = new NamazuBlobRepository({ settingsRepository });
    // act
    const actual = sut.assetUrl(assetId);
    // assert
    expect(actual).toEqual(expected);
  });

  test('should produce a URL for the thumbnail', async function () {
    // arrange
    const assetId =
      'MjAxOC8wNS8zMS8yMTAwLzAxYng1enprYmthY3Rhdjl3ZXZnZW1tdnJ6LmpwZw==';
    const expected = 'http://example.com/thumbnail/480/320/' + assetId;
    const settingsRepository = new EnvSettingsRepository();
    settingsRepository.set('NAMAZU_URL', 'http://example.com');
    const sut = new NamazuBlobRepository({ settingsRepository });
    // act
    const actual = sut.thumbnailUrl(assetId, 480, 320);
    // assert
    expect(actual).toEqual(expected);
  });

  test('should produce a width-sized preview URL', async function () {
    const assetId =
      'MjAxOC8wNS8zMS8yMTAwLzAxYng1enprYmthY3Rhdjl3ZXZnZW1tdnJ6LmpwZw==';
    const settingsRepository = new EnvSettingsRepository();
    settingsRepository.set('NAMAZU_URL', 'http://example.com');
    const sut = new NamazuBlobRepository({ settingsRepository });
    expect(sut.previewUrl(assetId, { width: 800 })).toEqual(
      `http://example.com/preview/${assetId}?width=800`
    );
  });

  test('should produce a height-sized preview URL', async function () {
    const assetId =
      'MjAxOC8wNS8zMS8yMTAwLzAxYng1enprYmthY3Rhdjl3ZXZnZW1tdnJ6LmpwZw==';
    const settingsRepository = new EnvSettingsRepository();
    settingsRepository.set('NAMAZU_URL', 'http://example.com');
    const sut = new NamazuBlobRepository({ settingsRepository });
    expect(sut.previewUrl(assetId, { height: 600 })).toEqual(
      `http://example.com/preview/${assetId}?height=600`
    );
  });

  test('should store new file and get a 201 response', async function () {
    const relpath = '2018/05/31/2100/01bx5zzkbkactav9wevgemmvrz.jpg';
    const buf = Buffer.from(relpath, 'utf8');
    const key = buf.toString('base64url');
    const asset = new Asset(key);

    mockFetch(
      {
        url: `http://example.com/assets/${key}`,
        method: 'PUT',
        headers: {
          'Content-Type': 'image/jpeg'
        }
      },
      new Response('{"success": "true"}', {
        status: 201,
        statusText: 'Created'
      })
    );

    // copy test file to temporary path as it will be removed
    const tmpdir = temporaryDirectory();
    const incoming = path.join(tmpdir, 'fighting_kittens.jpg');
    await fs.copyFile('./test/fixtures/fighting_kittens.jpg', incoming);

    const settingsRepository = new EnvSettingsRepository();
    settingsRepository.set('NAMAZU_URL', 'http://example.com');
    const sut = new NamazuBlobRepository({ settingsRepository });
    await sut.storeBlob(incoming, asset);
  });

  test('should store new file and get a 409 response', async function () {
    const relpath = '2018/05/31/2100/01bx5zzkbkactav9wevgemmvrz.jpg';
    const buf = Buffer.from(relpath, 'utf8');
    const key = buf.toString('base64url');
    const asset = new Asset(key);

    mockFetch(
      {
        url: `http://example.com/assets/${key}`,
        method: 'PUT',
        headers: {
          'Content-Type': 'image/jpeg'
        }
      },
      new Response('{"success": "true"}', {
        status: 409,
        statusText: 'Created'
      })
    );

    // copy test file to temporary path as it will be removed
    const tmpdir = temporaryDirectory();
    const incoming = path.join(tmpdir, 'fighting_kittens.jpg');
    await fs.copyFile('./test/fixtures/fighting_kittens.jpg', incoming);

    const settingsRepository = new EnvSettingsRepository();
    settingsRepository.set('NAMAZU_URL', 'http://example.com');
    const sut = new NamazuBlobRepository({ settingsRepository });
    await sut.storeBlob(incoming, asset);
  });

  test('fetchMetadata normalizes image tags from namazu shape', async function () {
    const assetId = 'MjAxOC8wNS8zMS8yMTAwL2Zvby5qcGc';
    // Mirrors namazu's actual `/metadata` response: each tag is an object
    // with kamadak-exif's display `description` and the raw typed `value`.
    mockFetch(
      {
        url: `http://example.com/metadata/${assetId}`,
        method: 'GET'
      },
      Response.json({
        Make: {
          description: '"EASTMAN KODAK COMPANY"',
          value: ['EASTMAN KODAK COMPANY']
        },
        Model: {
          description: '"KODAK DC280 ZOOM DIGITAL CAMERA"',
          value: ['KODAK DC280 ZOOM DIGITAL CAMERA']
        },
        DateTimeOriginal: {
          description: '2003-09-03 17:24:35',
          value: ['2003:09:03 17:24:35']
        },
        ExposureTime: {
          description: '1/125 s',
          value: [[1, 125]]
        },
        FNumber: {
          description: 'f/9.5',
          value: [[95, 10]]
        },
        Orientation: {
          description: 'row 0 at top and column 0 at left',
          value: [1]
        }
      })
    );

    const settingsRepository = new EnvSettingsRepository();
    settingsRepository.set('NAMAZU_URL', 'http://example.com');
    const sut = new NamazuBlobRepository({ settingsRepository });
    const result: any = await sut.fetchMetadata(assetId, 'image/jpeg');

    // ASCII tags use raw value: no embedded quotes.
    expect(result.Make.description).toEqual('EASTMAN KODAK COMPANY');
    expect(result.Model.description).toEqual('KODAK DC280 ZOOM DIGITAL CAMERA');
    // DateTimeOriginal uses colon-separated format from raw value so
    // parseExifDate can read it.
    expect(result.DateTimeOriginal.description).toEqual('2003:09:03 17:24:35');
    // ExposureTime drops the trailing seconds unit.
    expect(result.ExposureTime.description).toEqual('1/125');
    // Non-ASCII, non-ExposureTime descriptions pass through unchanged.
    expect(result.FNumber.description).toEqual('f/9.5');
    expect(result.Orientation.description).toEqual(
      'row 0 at top and column 0 at left'
    );
    // Raw typed value is preserved on every tag.
    expect(result.FNumber.value).toEqual([[95, 10]]);
    expect(result.Orientation.value).toEqual([1]);
  });

  test('fetchMetadata passes video metadata through unchanged', async function () {
    const assetId = 'MjAxOS0wNC0xNS8wODMwL3ZpZGVvLm1wNA';
    const videoMeta = {
      streams: [
        { codec_type: 'video', codec_name: 'h264', width: 1920, height: 1080 }
      ],
      format: { duration: '12.5' }
    };
    mockFetch(
      {
        url: `http://example.com/metadata/${assetId}`,
        method: 'GET'
      },
      Response.json(videoMeta)
    );

    const settingsRepository = new EnvSettingsRepository();
    settingsRepository.set('NAMAZU_URL', 'http://example.com');
    const sut = new NamazuBlobRepository({ settingsRepository });
    const result = await sut.fetchMetadata(assetId, 'video/mp4');
    expect(result).toEqual(videoMeta);
  });

  test('fetchMetadata returns null on 204', async function () {
    const assetId = 'MjAxOS0wNC0xNS8wODMwL2VtcHR5LnR4dA';
    mockFetch(
      {
        url: `http://example.com/metadata/${assetId}`,
        method: 'GET'
      },
      new Response(null, { status: 204 })
    );
    const settingsRepository = new EnvSettingsRepository();
    settingsRepository.set('NAMAZU_URL', 'http://example.com');
    const sut = new NamazuBlobRepository({ settingsRepository });
    const result = await sut.fetchMetadata(assetId, 'image/jpeg');
    expect(result).toBeNull();
  });

  test('should delete file and get a 200 response', async function () {
    const relpath = '2018/05/31/2100/01bx5zzkbkactav9wevgemmvrz.jpg';
    const buf = Buffer.from(relpath, 'utf8');
    const key = buf.toString('base64url');

    mockFetch(
      {
        url: `http://example.com/assets/${key}`,
        method: 'DELETE'
      },
      new Response('', {
        status: 200,
        statusText: 'OK'
      })
    );

    const settingsRepository = new EnvSettingsRepository();
    settingsRepository.set('NAMAZU_URL', 'http://example.com');
    const sut = new NamazuBlobRepository({ settingsRepository });
    await sut.deleteBlob(key);
  });
});
