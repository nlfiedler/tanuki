//
// Copyright (c) 2025 Nathan Fiedler
//
import { Asset } from 'tanuki/server/domain/entities/Asset.ts';

/**
 * Repository for managing the asset file content.
 */
interface BlobRepository {

  /**
   * Move the given file into the blob store. Existing blobs will not be
   * overwritten.
   *
   * @param filepath - path to the incoming asset.
   * @param asset - asset entity.
   * @param asset.key - unique asset identifier.
   */
  storeBlob(filepath: string, asset: Asset): Promise<void>;

  /**
   * Move the given file into the blob store, replacing whatever is already
   * there. Used when an asset is to be replaced by a different version.
   *
   * @param filepath - path to the incoming asset.
   * @param asset - asset entity.
   */
  replaceBlob(filepath: string, asset: Asset): Promise<void>;

  /**
   * Return the full path to the asset in blob storage.
   * 
   * @param assetId - unique asset identifier.
   * @returns path to the file.
   */
  blobPath(assetId: string): string;

  /**
   * Change the identity of the asset in blob storage.
   *
   * @param oldId - old asset identifier.
   * @param newId - new asset identifier.
   */
  renameBlob(oldId: string, newId: string): Promise<void>;

  /**
   * Produce a thumbnail of the desired size for the asset.
   *
   * @param width - width in pixels for the thumbnail.
   * @param height - height in pixels for the thumbnail.
   * @param assetId - unique asset identifier.
   * @returns buffer of raw bytes.
   */
  thumbnail(width: number, height: number, assetId: string): Promise<Buffer>;

  /**
   * Clear the thumbnail cache of any entries for the given asset.
   * 
   * @param assetId - unique asset identifier.
   */
  clearCache(assetId: string): Promise<void>;
}

export { type BlobRepository };
