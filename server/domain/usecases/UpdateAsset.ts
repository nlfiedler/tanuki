//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import { Asset } from 'tanuki/server/domain/entities/Asset.ts';
import { AssetInput } from 'tanuki/server/domain/entities/AssetInput.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/RecordRepository.ts';
import * as helpers from './helpers.ts';

export default ({ recordRepository }: { recordRepository: RecordRepository; }) => {
  assert.ok(recordRepository, 'record repository must be defined');
  /**
   * Return all of the unique locations in the record repository.
   * 
   * @returns list of unique location records.
   */
  return async (assetInput: AssetInput): Promise<Asset> => {
    // fetch existing record to merge with incoming values
    const asset = await recordRepository.getAssetById(assetInput.key);
    if (asset === null) {
      throw new Error('asset not found');
    }
    // merge the incoming values with the existing record
    mergeAssetInput(asset, assetInput);
    // store the updated record in the repository
    recordRepository.putAsset(asset);
    // self.cache.clear()?;
    return asset;
  };
};

/** Modify the given asset using the values from the asset input. */
function mergeAssetInput(asset: Asset, assetInput: AssetInput) {
  if (Array.isArray(assetInput.tags)) {
    // incoming tags replace existing tags, even if the are none
    const nonEmpty = assetInput.tags.map((e: string) => e.trim()).filter((e: string) => e.length > 0);
    asset.tags = Array.from<string>(new Set(nonEmpty)).sort();
  }
  if (assetInput.filename && assetInput.filename.length > 0) {
    asset.filename = assetInput.filename;
  }
  // merge the existing and new location, if any, and save if changed
  let location = helpers.mergeLocations(asset.location, assetInput.location);
  if (location?.hasValues()) {
    asset.location = location;
  }
  // parse the caption to glean location and additional tags
  if (assetInput.caption) {
    const { tags, location } = helpers.parseCaption(assetInput.caption);
    // tags in the caption are merged with the asset/input tags
    const alltags = asset.tags.concat(tags);
    asset.tags = Array.from<string>(new Set(alltags)).sort();
    if (!asset.location?.hasValues()) {
      // do not overwrite current location if it is already set
      asset.location = location;
    }
  }
  // custom date/time can be updated but never cleared, otherwise it is
  // impossible to update the asset without always setting the date/time
  if (assetInput.datetime) {
    asset.userDate = assetInput.datetime;
  }
  // do not overwrite media_type with null/blank values
  if (assetInput.mediaType && assetInput.mediaType.length > 0) {
    asset.mediaType = assetInput.mediaType.toLowerCase();
  }
}
