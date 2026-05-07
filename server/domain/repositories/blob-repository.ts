//
// Copyright (c) 2025 Nathan Fiedler
//
import { Asset } from 'tanuki/server/domain/entities/asset.ts';

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
   * Delete the blob associated with the given asset identifier.
   *
   * @param assetId - asset identifier.
   */
  deleteBlob(assetId: string): Promise<void>;

  /**
   * Read a byte range from the asset blob. Returns however many bytes are
   * actually available within the requested range; an empty buffer means
   * the start offset is at or past the end of the blob.
   *
   * @param assetId - unique asset identifier.
   * @param start - inclusive zero-based start offset.
   * @param end - inclusive end offset.
   * @returns buffer containing the bytes that were read.
   */
  fetchRange(assetId: string, start: number, end: number): Promise<Buffer>;

  /**
   * Return the URL for fetching the asset.
   *
   * @param assetId - unique asset identifier.
   * @returns URL from which to GET the asset.
   */
  assetUrl(assetId: string): string;

  /**
   * Return a URL for producing a thumbnail of the asset.
   *
   * @param assetId - unique asset identifier.
   * @param width - width in pixels for the thumbnail.
   * @param height - height in pixels for the thumbnail.
   * @returns URL from which to GET the thumbnail.
   */
  thumbnailUrl(assetId: string, width: number, height: number): string;

  /**
   * Return a URL for producing an aspect-preserving preview of the asset,
   * sized by either width or height (exactly one).
   *
   * @param assetId - unique asset identifier.
   * @param opts - either `{ width }` or `{ height }` in pixels.
   * @returns URL from which to GET the preview.
   */
  previewUrl(
    assetId: string,
    opts: { width: number } | { height: number }
  ): string;
}

export { type BlobRepository };
