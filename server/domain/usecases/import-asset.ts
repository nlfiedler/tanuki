//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import fs from 'node:fs/promises';
import * as helpers from './helpers.ts';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { type BlobRepository } from 'tanuki/server/domain/repositories/blob-repository.ts';
import { type LocationRepository } from 'tanuki/server/domain/repositories/location-repository.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';

export default ({
  recordRepository,
  blobRepository,
  locationRepository
}: {
  recordRepository: RecordRepository;
  blobRepository: BlobRepository;
  locationRepository: LocationRepository;
}) => {
  assert.ok(recordRepository, 'record repository must be defined');
  assert.ok(blobRepository, 'blob repository must be defined');
  assert.ok(locationRepository, 'location repository must be defined');
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

      // some applications (e.g. Photos.app) will set the file modified time
      // appropriately, so if the asset itself does not have an original
      // date/time, use that
      const exifDate = await helpers.getOriginalDate(mimetype, filepath);
      const originalDate = exifDate === null ? modified : new Date(exifDate);
      asset = new Asset(assetId);
      asset.checksum = digest;
      asset.filename = originalname;
      asset.byteLength = length;
      asset.mediaType = mimetype;
      asset.importDate = now;
      asset.originalDate = originalDate;

      // attempt to fill in the city and region if the asset has GPS coordinates
      // and a reverse gecoding location repository has been configured
      const coords = await helpers.getCoordinates(mimetype, filepath);
      if (coords) {
        const geocoded = await locationRepository.findLocation(coords);
        if (geocoded !== null) {
          asset.location = helpers.locationFromGeocoded(geocoded);
        }
      }
      await recordRepository.putAsset(asset);
    }
    // blob repo will ensure the temporary file is (re)moved
    await blobRepository.storeBlob(filepath, asset);
    return asset;
  };
};
