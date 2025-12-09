//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import { Asset } from 'tanuki/server/domain/entities/Asset.ts';
import { Location } from 'tanuki/server/domain/entities/Location.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/RecordRepository.ts';

export default ({
  recordRepository
}: {
  recordRepository: RecordRepository;
}) => {
  assert.ok(recordRepository, 'record repository must be defined');
  /**
   * Store the given assets (formatted as "dump" records) into the repository.
   *
   * @param incoming - assets in "dump" format.
   */
  return async (incoming: Array<any>) => {
    // convert the dump records into Asset entities
    const assets = incoming.map((r) => {
      const asset = new Asset(r.key);
      asset.setChecksum(r.checksum);
      asset.setFilename(r.filename);
      asset.setByteLength(r.byte_length);
      asset.setMediaType(r.media_type);
      asset.setTags(r.tags);
      asset.setImportDate(new Date(r.import_date));
      asset.setCaption(r.caption);
      if (r.location) {
        // Location gets special treatment in which values that have only a
        // label are written to the dump file as just a string.
        if (typeof r.location === 'string') {
          asset.setLocation(new Location(r.location));
        } else {
          asset.setLocation(
            Location.fromRaw(r.location.l, r.location.c, r.location.r)
          );
        }
      }
      if (r.user_date) {
        asset.setUserDate(new Date(r.user_date));
      }
      if (r.original_date) {
        asset.setOriginalDate(new Date(r.original_date));
      }
      return asset;
    });
    if (assets.length > 0) {
      await recordRepository.storeAssets(assets);
    }
  };
};
