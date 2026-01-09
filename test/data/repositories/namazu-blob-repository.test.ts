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

  test('should store new file and get a 201 response', async function () {
    const relpath = '2018/05/31/2100/01bx5zzkbkactav9wevgemmvrz.jpg';
    const buf = Buffer.from(relpath, 'utf8');
    const key = buf.toString('base64');
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
    const key = buf.toString('base64');
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

  test('should delete file and get a 200 response', async function () {
    const relpath = '2018/05/31/2100/01bx5zzkbkactav9wevgemmvrz.jpg';
    const buf = Buffer.from(relpath, 'utf8');
    const key = buf.toString('base64');

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
