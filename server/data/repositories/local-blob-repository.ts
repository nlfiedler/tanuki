//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import fs from 'node:fs/promises';
import path from 'node:path';
import ExifReader from 'exifreader';
import sharp from 'sharp';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { type BlobRepository } from 'tanuki/server/domain/repositories/blob-repository.ts';
import { type SettingsRepository } from 'tanuki/server/domain/repositories/settings-repository.ts';
import { runFfprobe, stripRawImageTags } from 'tanuki/server/domain/usecases/helpers.ts';

/**
 * Blob repository that stores assets in attached disk storage.
 */
class LocalBlobRepository implements BlobRepository {
  basepath: string;

  constructor({
    settingsRepository
  }: {
    settingsRepository: SettingsRepository;
  }) {
    this.basepath = settingsRepository.get('ASSETS_PATH');
    assert.ok(this.basepath, 'missing ASSETS_PATH environment variable');
  }

  /** Convert the asset identifier to the full path of the asset. */
  blobPath(assetId: string): string {
    const buf = Buffer.from(assetId, 'base64url');
    const relpath = buf.toString('utf8');
    return path.join(this.basepath, relpath);
  }

  /** @inheritdoc */
  async storeBlob(filepath: string, asset: Asset) {
    const destpath = this.blobPath(asset.key);
    // do not overwrite existing asset blobs
    if (!(await accessible(destpath))) {
      const parent = path.dirname(destpath);
      await fs.mkdir(parent, { recursive: true });
      // use copy to handle crossing file systems
      await fs.copyFile(filepath, destpath);
      // ensure file is readable for backup programs and the like
      await fs.chmod(destpath, '0644');
    }
    await fs.rm(filepath);
  }

  /** @inheritdoc */
  async deleteBlob(assetId: string) {
    const filepath = this.blobPath(assetId);
    await fs.rm(filepath);
  }

  /** @inheritdoc */
  async fetchRange(
    assetId: string,
    start: number,
    end: number
  ): Promise<Buffer> {
    const filepath = this.blobPath(assetId);
    const length = Math.max(0, end - start + 1);
    const handle = await fs.open(filepath, 'r');
    try {
      const view = new Uint8Array(length);
      const { bytesRead } = await handle.read(view, 0, length, start);
      return Buffer.from(view.buffer, 0, bytesRead);
    } finally {
      await handle.close();
    }
  }

  /** @inheritdoc */
  assetUrl(assetId: string): string {
    // served by an endpoint defined in preso/routes/assets.ts
    return `/assets/raw/${assetId}`;
  }

  /** @inheritdoc */
  thumbnailUrl(assetId: string, width: number, height: number): string {
    // served by an endpoint defined in preso/routes/assets.ts
    return `/assets/thumbnail/${width}/${height}/${assetId}`;
  }

  /** @inheritdoc */
  previewUrl(
    assetId: string,
    opts: { width: number } | { height: number }
  ): string {
    const param =
      'width' in opts ? `width=${opts.width}` : `height=${opts.height}`;
    return `/assets/preview/${assetId}?${param}`;
  }

  /** @inheritdoc */
  async fetchMetadata(
    assetId: string,
    mediaType: string
  ): Promise<object | null> {
    const filepath = this.blobPath(assetId);
    if (mediaType.startsWith('image/')) {
      try {
        const tags = await ExifReader.load(filepath);
        return stripRawImageTags(tags);
      } catch {
        return imageDimensionTags(filepath);
      }
    }
    if (mediaType.startsWith('video/')) {
      return runFfprobe(filepath);
    }
    return null;
  }
}

/**
 * Fallback for images lacking an EXIF header: probe the raw pixel dimensions
 * with sharp and return them shaped like ExifReader tags so downstream
 * consumers (parseImageTags, etc.) can read them via the usual keys.
 */
async function imageDimensionTags(
  filepath: string
): Promise<object | null> {
  try {
    const { width, height } = await sharp(filepath).metadata();
    if (typeof width !== 'number' || typeof height !== 'number') return null;
    return {
      PixelXDimension: { value: width, description: String(width) },
      PixelYDimension: { value: height, description: String(height) }
    };
  } catch {
    return null;
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
    await fs.access(path);
    return true;
  } catch {
    return false;
  }
}

export { LocalBlobRepository, accessible };
