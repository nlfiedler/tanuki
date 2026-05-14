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
   * Walk every asset and, for video assets that lack persisted metadata,
   * fetch the raw ffprobe output from the blob store and persist parsed
   * metadata.
   *
   * @returns the number of assets that were updated.
   */
  return async (): Promise<number> => {
    let updated = 0;
    let cursor = null;
    while (true) {
      const [assets, next] = await recordRepository.fetchAssets(cursor, 1024);
      if (assets.length === 0) break;
      for (const asset of assets) {
        if (!asset.mediaType.startsWith('video/')) continue;
        if (asset.metadata !== null) continue;
        try {
          const raw = await blobRepository.fetchMetadata(
            asset.key,
            asset.mediaType
          );
          if (raw === null) continue;
          const { metadata } = helpers.parseVideoMetadata(raw);
          if (metadata.hasValues()) {
            asset.metadata = metadata;
            await recordRepository.putAsset(asset);
            updated++;
          }
        } catch {
          // skip this asset; missing or unreadable blob is non-fatal
        }
      }
      cursor = next;
    }
    return updated;
  };
};
