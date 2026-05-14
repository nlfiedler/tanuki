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
   * Re-parse the `originalDate` for every image asset using the EXIF
   * `OffsetTimeOriginal` tag when available. Existing dates were stored with
   * `Date.UTC(...)` applied to local-clock EXIF fields, so any image whose
   * camera reported a timezone offset has an `originalDate` that is off by
   * that offset. This use case updates those records to the correct UTC
   * instant.
   *
   * Only updates records where the recomputed value differs from the stored
   * one — assets without `OffsetTimeOriginal` (or already correct) are left
   * alone.
   *
   * @returns the number of assets whose originalDate was changed.
   */
  return async (): Promise<number> => {
    let updated = 0;
    let cursor = null;
    while (true) {
      const [assets, next] = await recordRepository.fetchAssets(cursor, 1024);
      if (assets.length === 0) break;
      for (const asset of assets) {
        if (!asset.mediaType.startsWith('image/')) continue;
        try {
          const raw: any = await blobRepository.fetchMetadata(
            asset.key,
            asset.mediaType
          );
          if (raw === null) continue;
          const info = helpers.parseImageTags(raw);
          if (info.originalDate === null) continue;
          // Only rewrite when the recomputed value actually differs.
          const current = asset.originalDate?.getTime() ?? null;
          if (current === info.originalDate) continue;
          asset.originalDate = new Date(info.originalDate);
          await recordRepository.putAsset(asset);
          updated++;
        } catch {
          // skip on error
        }
      }
      cursor = next;
    }
    return updated;
  };
};
