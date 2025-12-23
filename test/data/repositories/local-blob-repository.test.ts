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
} from 'tanuki/server/data/repositories/local-bob-repository.ts';

describe('LocalBlobRepository', function () {
  test('should decode the asset key into a path', async function () {
    // arrange
    const expected = path.normalize('foo/2018/05/31/2100/01bx5zzkbkactav9wevgemmvrz.jpg');
    const encoded =
      'MjAxOC8wNS8zMS8yMTAwLzAxYng1enprYmthY3Rhdjl3ZXZnZW1tdnJ6LmpwZw==';
    const settingsRepository = new EnvSettingsRepository();
    settingsRepository.set('ASSETS_PATH', 'foo');
    const sut = new LocalBlobRepository({ settingsRepository });
    // act
    const actual = sut.blobPath(encoded);
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
});
