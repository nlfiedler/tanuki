//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import fs from 'node:fs/promises';
import * as helpers from './helpers.ts';
import logger from 'tanuki/server/logger.ts';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { type BlobRepository } from 'tanuki/server/domain/repositories/blob-repository.ts';
import { type FaceStore } from 'tanuki/server/domain/repositories/face-store.ts';
import { type LocationRepository } from 'tanuki/server/domain/repositories/location-repository.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';
import { type SearchRepository } from 'tanuki/server/domain/repositories/search-repository.ts';

/** Priority for live-import jobs, so they preempt backfill (priority 0). */
const LIVE_IMPORT_PRIORITY = 10;

export default ({
  recordRepository,
  blobRepository,
  locationRepository,
  searchRepository,
  faceStore
}: {
  recordRepository: RecordRepository;
  blobRepository: BlobRepository;
  locationRepository: LocationRepository;
  searchRepository: SearchRepository;
  faceStore: FaceStore;
}) => {
  assert.ok(recordRepository, 'record repository must be defined');
  assert.ok(blobRepository, 'blob repository must be defined');
  assert.ok(locationRepository, 'location repository must be defined');
  assert.ok(searchRepository, 'search repository must be defined');
  assert.ok(faceStore, 'face store must be defined');
  /**
   * Import the given file into the system as a new asset, moving the file into
   * the blob storage and creating a new record in the repository.
   *
   * @param filepath - path of file to be imported.
   * @param originalname - name of the file that was uploaded.
   * @param mimetype - media type of the file.
   * @param modified - date/time when file was last modified.
   * @returns asset entity.
   */
  return async (
    filepath: string,
    originalname: string,
    mimetype: string,
    modified: Date
  ): Promise<Asset> => {
    const digest = await helpers.checksumFile(filepath);
    let asset = await recordRepository.getAssetByDigest(digest);
    if (asset === null) {
      const now = new Date();
      const assetId = helpers.newAssetId(now, mimetype);
      // eslint-disable-next-line unicorn/no-await-expression-member
      const length = (await fs.stat(filepath)).size;

      // Single-pass extraction: one EXIF read for images (date, coords,
      // metadata) or one ffprobe call for videos. Either may return null for
      // unsupported / unparseable files; in that case fall back to the file's
      // modified time for the original date.
      let extractedDate: number | null = null;
      let coords = null;
      let metadata = null;
      if (mimetype.startsWith('image/')) {
        const info = await helpers.extractImageInfo(filepath);
        if (info) {
          extractedDate = info.originalDate;
          coords = info.coordinates;
          metadata = info.metadata;
        }
      } else if (mimetype.startsWith('video/')) {
        const info = await helpers.extractVideoInfo(filepath);
        if (info) {
          extractedDate = info.originalDate;
          metadata = info.metadata;
        }
      }

      // some applications (e.g. Photos.app) will set the file modified time
      // appropriately, so if the asset itself does not have an original
      // date/time, use that
      const originalDate =
        extractedDate === null ? modified : new Date(extractedDate);
      asset = new Asset(assetId);
      asset.checksum = digest;
      asset.filename = originalname;
      asset.byteLength = length;
      asset.mediaType = mimetype;
      asset.importDate = now;
      asset.originalDate = originalDate;
      asset.metadata = metadata && metadata.hasValues() ? metadata : null;

      // attempt to fill in the city and region if the asset has GPS coordinates
      // and a reverse gecoding location repository has been configured
      if (coords) {
        const geocoded = await locationRepository.findLocation(coords);
        if (geocoded !== null) {
          asset.location = helpers.locationFromGeocoded(geocoded);
        }
      }
      await recordRepository.putAsset(asset);
      // blob repository will ensure the temporary file is (re)moved
      await blobRepository.storeBlob(filepath, asset);
      await searchRepository.clear();

      // Queue background synthetic-data extraction for images: one job per
      // kind, both at live-import priority so they preempt backfill. A queue
      // hiccup must not fail the import — the asset is already stored, and the
      // backfill mutations can pick it up later.
      if (mimetype.startsWith('image/')) {
        for (const kind of ['labels', 'faces'] as const) {
          try {
            await faceStore.enqueueJob(asset.key, kind, LIVE_IMPORT_PRIORITY);
          } catch (error: any) {
            logger.warn(`import-asset: failed to enqueue ${kind} job:`, error);
          }
        }
      }
    } else {
      // remove the temporary file
      await fs.rm(filepath);
    }
    return asset;
  };
};
