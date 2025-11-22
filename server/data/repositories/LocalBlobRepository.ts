//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert'
import fs from 'node:fs/promises'
import path from 'node:path'
import { Asset } from 'tanuki/server/domain/entities/Asset.ts';
import { type BlobRepository } from 'tanuki/server/domain/repositories/BlobRepository.ts'
import { type SettingsRepository } from 'tanuki/server/domain/repositories/SettingsRepository.ts';

/* eslint-disable no-unused-vars */

/**
 * Blob repository that stores assets in attached disk storage.
 */
class LocalBlobRepository implements BlobRepository {
  basepath: string;

  constructor({ settingsRepository }: { settingsRepository: SettingsRepository; }) {
    this.basepath = settingsRepository.get('ASSETS_PATH')
    assert.ok(this.basepath, 'missing ASSETS_PATH environment variable')
  }

  /** @inheritdoc */
  async storeBlob(filepath: string, asset: Asset) {
    const destpath = this.blobPath(asset.key)
    // do not overwrite existing asset blobs
    if (!(await accessible(destpath))) {
      const parent = path.dirname(destpath)
      await fs.mkdir(parent, { recursive: true })
      // use copy to handle crossing file systems
      await fs.copyFile(filepath, destpath)
      // ensure file is readable for backup programs and the like
      await fs.chmod(destpath, '0644')
    }
    await fs.rm(filepath)
  }

  /** @inheritdoc */
  replaceBlob(filepath: string, asset: Asset) {
    return Promise.reject(new Error('not implemented'))
  }

  /** @inheritdoc */
  blobPath(assetId: string) {
    const buf = Buffer.from(assetId, 'base64')
    const relpath = buf.toString('utf8')
    return path.join(this.basepath, relpath)
  }

  /** @inheritdoc */
  renameBlob(oldId: string, newId: string) {
    return Promise.reject(new Error('not implemented'))
  }

  /** @inheritdoc */
  thumbnail(width: number, height: number, assetId: string) {
    return Promise.reject(new Error('not implemented'))
  }

  /** @inheritdoc */
  clearCache(assetId: string) {
    return Promise.reject(new Error('not implemented'))
  }
}

/**
 * Determine if the given file path actually exists..
 *
 * @param path - path of file to be checked.
 * @returns true if file is accessible, false otherwise.
 */
async function accessible(path: string): Promise<boolean> {
  try {
    await fs.access(path)
    return true
  } catch {
    return false
  }
}

export { LocalBlobRepository, accessible }
