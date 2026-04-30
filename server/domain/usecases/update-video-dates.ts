//
// Copyright (c) 2026 Nathan Fiedler
//
import assert from 'node:assert';
import { type BlobRepository } from 'tanuki/server/domain/repositories/blob-repository.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';
import * as helpers from './helpers.ts';

export default ({
  recordRepository,
  blobRepository
}: {
  recordRepository: RecordRepository;
  blobRepository: BlobRepository;
}) => {
  assert.ok(recordRepository, 'record repository must be defined');
  assert.ok(blobRepository, 'blob repository must be defined');
  /**
   * One-shot backfill: walk every asset, and for each video that lacks an
   * `originalDate`, read just enough bytes of the blob to parse its mp4/mov
   * creation time and persist the updated record.
   *
   * @returns the number of assets that were updated.
   */
  return async (): Promise<number> => {
    let updated = 0;
    let cursor = null;
    while (true) {
      const [assets, next] = await recordRepository.fetchAssets(cursor, 1024);
      if (assets.length === 0) {
        break;
      }
      for (const asset of assets) {
        if (!asset.mediaType.startsWith('video/')) continue;
        if (asset.originalDate !== null) continue;
        try {
          const millis = await helpers.getCreationTimeFromBlob((start, end) =>
            blobRepository.fetchRange(asset.key, start, end)
          );
          if (millis !== null) {
            asset.setOriginalDate(new Date(millis));
            await recordRepository.putAsset(asset);
            updated++;
          }
        } catch {
          // skip this asset; a missing or unreadable blob is non-fatal
        }
      }
      cursor = next;
    }
    return updated;
  };
};
