//
// Copyright (c) 2025 Nathan Fiedler
//
import fs from 'node:fs/promises';
import path from 'node:path';
import { describe, expect, test } from 'bun:test';
import { temporaryDirectory } from 'tempy';
// prepare the test environment as early as possible
import 'tanuki/test/env.ts';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { EnvSettingsRepository } from 'tanuki/server/data/repositories/env-settings-repository.ts';
import {
  LocalBlobRepository,
  accessible
} from 'tanuki/server/data/repositories/local-blob-repository.ts';

describe('LocalBlobRepository', function () {
  test('should produce a URL for the full asset', async function () {
    // arrange
    const assetId =
      'MjAxOC8wNS8zMS8yMTAwLzAxYng1enprYmthY3Rhdjl3ZXZnZW1tdnJ6LmpwZw==';
    const expected = path.normalize('/assets/raw/' + assetId);
    const settingsRepository = new EnvSettingsRepository();
    settingsRepository.set('ASSETS_PATH', 'ignored');
    const sut = new LocalBlobRepository({ settingsRepository });
    // act
    const actual = sut.assetUrl(assetId);
    // assert
    expect(actual).toEqual(expected);
  });

  test('should produce a URL for the thumbnail', async function () {
    // arrange
    const assetId =
      'MjAxOC8wNS8zMS8yMTAwLzAxYng1enprYmthY3Rhdjl3ZXZnZW1tdnJ6LmpwZw==';
    const expected = path.normalize('/assets/thumbnail/480/320/' + assetId);
    const settingsRepository = new EnvSettingsRepository();
    settingsRepository.set('ASSETS_PATH', 'ignored');
    const sut = new LocalBlobRepository({ settingsRepository });
    // act
    const actual = sut.thumbnailUrl(assetId, 480, 320);
    // assert
    expect(actual).toEqual(expected);
  });

  test('should move a new file into the blob store', async function () {
    // arrange
    const relpath = '2018/05/31/2100/01bx5zzkbkactav9wevgemmvrz.jpg';
    const buf = Buffer.from(relpath, 'utf8');
    const key = buf.toString('base64');
    const asset = new Asset(key);
    const tmpdir = temporaryDirectory();
    // copy test file to temporary path as it will be (re)moved
    const incoming = path.join(tmpdir, 'fighting_kittens.jpg');
    await fs.copyFile('./test/fixtures/fighting_kittens.jpg', incoming);
    // act
    const basepath = path.join(tmpdir, 'blobs');
    const settingsRepository = new EnvSettingsRepository();
    settingsRepository.set('ASSETS_PATH', basepath);
    const sut = new LocalBlobRepository({ settingsRepository });
    await sut.storeBlob(incoming, asset);
    // assert
    await fs.access(sut.blobPath(key));
    expect(await accessible(incoming)).toBeFalse();
  });

  test('should delete a file from the blob store', async function () {
    // arrange
    const relpath = '2018/05/31/2100/01bx5zzkbkactav9wevgemmvrz.jpg';
    const buf = Buffer.from(relpath, 'utf8');
    const key = buf.toString('base64');
    const tmpdir = temporaryDirectory();
    // copy test file to blob store path as it will be (re)moved
    const filepath = path.join(tmpdir, 'blobs', relpath);
    await fs.mkdir(path.dirname(filepath), { recursive: true });
    await fs.copyFile('./test/fixtures/fighting_kittens.jpg', filepath);
    // act
    const basepath = path.join(tmpdir, 'blobs');
    const settingsRepository = new EnvSettingsRepository();
    settingsRepository.set('ASSETS_PATH', basepath);
    const sut = new LocalBlobRepository({ settingsRepository });
    await sut.deleteBlob(key);
    // assert
    expect(await accessible(sut.blobPath(key))).toBeFalse();
  });
});
