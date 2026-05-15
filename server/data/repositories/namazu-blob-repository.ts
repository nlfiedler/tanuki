//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import fs from 'node:fs/promises';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { type BlobRepository } from 'tanuki/server/domain/repositories/blob-repository.ts';
import { type SettingsRepository } from 'tanuki/server/domain/repositories/settings-repository.ts';

/**
 * Blob repository that stores assets in the namazu remote blob store.
 */
class NamazuBlobRepository implements BlobRepository {
  baseurl: string;

  constructor({
    settingsRepository
  }: {
    settingsRepository: SettingsRepository;
  }) {
    this.baseurl = settingsRepository.get('NAMAZU_URL').replace(/\/+$/, '');
    assert.ok(this.baseurl, 'missing NAMAZU_URL environment variable');
  }

  makeAssetUrl(assetId: string): string {
    return this.baseurl + '/assets/' + assetId;
  }

  /** @inheritdoc */
  async storeBlob(filepath: string, asset: Asset) {
    const url = this.makeAssetUrl(asset.key);
    let retries = 0;
    while (retries < 10) {
      try {
        const file = Bun.file(filepath);
        const request = new Request(url, {
          method: 'PUT',
          headers: {
            'Content-Type': file.type || 'application/octet-stream',
            Connection: 'close'
          },
          body: file
        });
        const response = await fetch(request);
        if (response.status !== 201 && response.status !== 409) {
          throw new Error('expected 201 or 409 response');
        }
        break;
      } catch (error: any) {
        // some errors are temporary, retry after waiting briefly
        if (
          error.code !== 'EAGAIN' &&
          error.code !== 'ENOTCONN' &&
          error.code !== 'EPIPE'
        ) {
          throw error;
        }
        await Bun.sleep(retries * 100);
      }
      retries++;
    }
    await fs.rm(filepath);
  }

  /** @inheritdoc */
  async deleteBlob(assetId: string) {
    const url = this.makeAssetUrl(assetId);
    const request = new Request(url, { method: 'DELETE' });
    const response = await fetch(request);
    if (!response.ok) {
      throw new Error(response.statusText);
    }
  }

  /** @inheritdoc */
  async fetchRange(
    assetId: string,
    start: number,
    end: number
  ): Promise<Buffer> {
    const url = this.makeAssetUrl(assetId);
    const response = await fetch(url, {
      headers: { Range: `bytes=${start}-${end}` }
    });
    if (response.status === 416) {
      // requested range is past the end of the blob
      return Buffer.alloc(0);
    }
    // 206 is the expected partial-content response; 200 can occur if the
    // server chose to ignore the Range header (e.g. for very small blobs),
    // in which case we slice the full body down to the requested range
    if (response.status !== 206 && response.status !== 200) {
      throw new Error(`unexpected ${response.status} fetching range`);
    }
    const body = Buffer.from(await response.arrayBuffer());
    if (response.status === 200) {
      return body.subarray(start, Math.min(body.length, end + 1));
    }
    return body;
  }

  /** @inheritdoc */
  assetUrl(assetId: string): string {
    return this.makeAssetUrl(assetId);
  }

  /** @inheritdoc */
  thumbnailUrl(assetId: string, width: number, height: number): string {
    return `${this.baseurl}/thumbnail/${width}/${height}/${assetId}`;
  }

  /** @inheritdoc */
  previewUrl(
    assetId: string,
    opts: { width: number } | { height: number }
  ): string {
    const param =
      'width' in opts ? `width=${opts.width}` : `height=${opts.height}`;
    return `${this.baseurl}/preview/${assetId}?${param}`;
  }

  /** @inheritdoc */
  async fetchMetadata(
    assetId: string,
    mediaType: string
  ): Promise<object | null> {
    const url = `${this.baseurl}/metadata/${assetId}`;
    let response: Response;
    try {
      response = await fetch(url);
    } catch {
      return null;
    }
    // 204 = extractor not applicable (or empty). Treat as null.
    if (response.status === 204) return null;
    if (!response.ok) return null;
    let json: any;
    try {
      json = await response.json();
    } catch {
      return null;
    }
    if (mediaType.startsWith('image/')) {
      return normalizeNamazuImageTags(json);
    }
    return json;
  }
}

// Reconcile namazu's `/metadata` output with the ExifReader-shaped tag map
// that `parseImageTags` reads. Namazu (via kamadak-exif) renders ASCII
// tags inside literal double quotes and uses hyphens in DateTime display
// strings, and appends units like " s" to ExposureTime — none of which
// match what ExifReader produces. Fortunately each tag also carries the
// raw typed `value`, so we can substitute the unquoted/raw form there.
function normalizeNamazuImageTags(tags: any): any {
  if (!tags || typeof tags !== 'object') return tags;
  const out: any = {};
  for (const [key, entry] of Object.entries(tags)) {
    if (
      entry &&
      typeof entry === 'object' &&
      'description' in entry &&
      'value' in entry
    ) {
      const e = entry as { description: string; value: unknown };
      let description = e.description;
      if (
        Array.isArray(e.value) &&
        e.value.length > 0 &&
        e.value.every((v) => typeof v === 'string')
      ) {
        description = e.value[0] as string;
      } else if (key === 'ExposureTime' && description.endsWith(' s')) {
        description = description.slice(0, -2);
      }
      out[key] = { ...e, description };
    } else {
      out[key] = entry;
    }
  }
  return out;
}

export { NamazuBlobRepository };
