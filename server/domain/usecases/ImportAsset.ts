//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import fs from 'node:fs/promises';
import * as helpers from './helpers.ts';
import { Asset } from 'tanuki/server/domain/entities/Asset.ts';
import { type BlobRepository } from 'tanuki/server/domain/repositories/BlobRepository.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/RecordRepository.ts';

/**
 * Count the number of records in the record repository.
 *
 * @param filepath - path of file to be imported.
 * @param originalname - name of the file that was uploaded.
 * @param mimetype - media type of the file.
 * @param modified - date/time when file was last modified.
 * @returns asset entity.
 */
export default ({ recordRepository, blobRepository }: { recordRepository: RecordRepository, blobRepository: BlobRepository; }) => {
  assert.ok(recordRepository, 'record repository must be defined');
  assert.ok(blobRepository, 'blob repository must be defined');
  // eslint-disable-next-line no-unused-vars
  return async (filepath: string, originalname: string, mimetype: string, modified: Date): Promise<Asset> => {
    const digest = await helpers.checksumFile(filepath);
    let asset = await recordRepository.getAssetByDigest(digest);
    if (asset === null) {
      const now = new Date();
      const assetId = helpers.newAssetId(now, mimetype);
      const length = (await fs.stat(filepath)).size;

      // let location = match get_gps_coordinates(&params.media_type, &params.filepath) {
      //     Ok(coords) => match self.geocoder.find_location(&coords) {
      //         Ok(geoloc) => Some(super::convert_location(geoloc)),
      //         Err(err) => {
      //             error!("import: geocode error: {}", err);
      //             None
      //         }
      //     },
      //     _ => None,
      // };

      // some applications (e.g. Photos.app) will set the file modified time
      // appropriately, so if the asset itself does not have an original
      // date/time, use that
      const exifDate = await helpers.getOriginalDate(mimetype, filepath);
      const originalDate = exifDate !== null ? new Date(exifDate) : modified;
      asset = new Asset(assetId);
      asset.checksum = digest;
      asset.filename = originalname;
      asset.byteLength = length;
      asset.mediaType = mimetype;
      asset.importDate = now;
      asset.originalDate = originalDate;
      await recordRepository.putAsset(asset);
      //         self.cache.clear()?;
    }
    // blob repo will ensure the temporary file is (re)moved
    await blobRepository.storeBlob(filepath, asset);
    return asset;
  };
};
