//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import fs from 'node:fs/promises';
import path from 'node:path';
import mime from 'mime';
import { Asset } from 'tanuki/server/domain/entities/Asset.ts';

type ImportAssetFn = (filepath: string, originalname: string, mimetype: string, modified: Date) => Promise<Asset>;

export default ({ importAsset, }: { importAsset: ImportAssetFn; }) => {
  assert.ok(importAsset, 'importAsset usecase must be defined');
  /**
   * Import all of the files in the uploads directory.
   *
   * @param uploadsPath - path to the uploads directory.
   * @returns number of assets imported.
   */
  return async (uploadsPath: string): Promise<number> => {
    const files = await fs.readdir(uploadsPath);
    let count = 0;
    for (const file of files) {
      const fullpath = path.join(uploadsPath, file);
      const stats = await fs.stat(fullpath);
      if (stats.isFile()) {
        const extension = path.extname(file);
        let mediaType = 'application/octet-stream';
        if (extension.startsWith('.')) {
          const inferred = mime.getType(extension.slice(1));
          if (inferred) {
            mediaType = inferred;
          }
        }
        await importAsset(fullpath, file, mediaType, stats.mtime);
        count++;
      }
    }
    return count;
  };
};
