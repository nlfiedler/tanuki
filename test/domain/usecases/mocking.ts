//
// Copyright (c) 2025 Nathan Fiedler
//
import { mock } from "bun:test";
import { Asset } from 'tanuki/server/domain/entities/Asset.ts';
import { AttributeCount } from 'tanuki/server/domain/entities/AttributeCount.ts';
import { Location } from 'tanuki/server/domain/entities/Location.ts';
import { SearchResult } from 'tanuki/server/domain/entities/SearchResult.ts';
import { type BlobRepository } from 'tanuki/server/domain/repositories/BlobRepository.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/RecordRepository.ts';

/**
 * Helper for producing a mock record repository implementation. Any undefined
 * functions will return null, empty string, zero, etc.
 */
export function recordRepositoryMock(
  { countAssets = undefined,
    getAssetById = undefined,
    getAssetByDigest = undefined,
    allTags = undefined,
    allLocations = undefined,
    rawLocations = undefined,
    allYears = undefined,
    putAsset = undefined,
    queryByTags = undefined,
    queryByLocations = undefined,
    queryByMediaType = undefined,
    queryBeforeDate = undefined,
    queryAfterDate = undefined,
    queryDateRange = undefined,
    queryNewborn = undefined,
  }:
    {
      countAssets?: () => Promise<number>,
      getAssetById?: (assetId: string) => Promise<Asset | null>,
      getAssetByDigest?: (digest: string) => Promise<Asset | null>,
      allTags?: () => Promise<AttributeCount[]>,
      allLocations?: () => Promise<AttributeCount[]>,
      rawLocations?: () => Promise<Location[]>,
      allYears?: () => Promise<AttributeCount[]>,
      putAsset?: (asset: Asset) => Promise<void>,
      queryByTags?: (tags: String[]) => Promise<SearchResult[]>,
      queryByLocations?: (locations: string[]) => Promise<SearchResult[]>,
      queryByMediaType?: (media_type: string) => Promise<SearchResult[]>,
      queryBeforeDate?: (before: Date) => Promise<SearchResult[]>,
      queryAfterDate?: (after: Date) => Promise<SearchResult[]>,
      queryDateRange?: (after: Date, before: Date) => Promise<SearchResult[]>,
      queryNewborn?: (after: Date) => Promise<SearchResult[]>;
    }
): RecordRepository {
  const mockRecordRepository: RecordRepository = {
    countAssets: countAssets || mock(() => Promise.resolve(0)),
    getAssetById: getAssetById || mock(() => Promise.resolve(null)),
    getAssetByDigest: getAssetByDigest || mock(() => Promise.resolve(null)),
    allTags: allTags || mock(() => Promise.resolve([])),
    allLocations: allLocations || mock(() => Promise.resolve([])),
    rawLocations: rawLocations || mock(() => Promise.resolve([])),
    allYears: allYears || mock(() => Promise.resolve([])),
    putAsset: putAsset || mock(() => Promise.resolve()),
    queryByTags: queryByTags || mock(() => Promise.resolve([])),
    queryByLocations: queryByLocations || mock(() => Promise.resolve([])),
    queryByMediaType: queryByMediaType || mock(() => Promise.resolve([])),
    queryBeforeDate: queryBeforeDate || mock(() => Promise.resolve([])),
    queryAfterDate: queryAfterDate || mock(() => Promise.resolve([])),
    queryDateRange: queryDateRange || mock(() => Promise.resolve([])),
    queryNewborn: queryNewborn || mock(() => Promise.resolve([])),
  };
  return mockRecordRepository;
}

/**
 * Helper for producing a mock blob repository implementation. Any undefined
 * functions will return null, empty string, zero, etc.
 */
export function blobRepositoryMock(
  {
    storeBlob = undefined,
    replaceBlob = undefined,
    blobPath = undefined,
    renameBlob = undefined,
    thumbnail = undefined,
    clearCache = undefined,
  }:
    {
      storeBlob?: (filepath: string, asset: Asset) => Promise<void>,
      replaceBlob?: (filepath: string, asset: Asset) => Promise<void>,
      blobPath?: (assetId: string) => string,
      renameBlob?: (oldId: string, newId: string) => Promise<void>,
      thumbnail?: (assetId: string, width: number, height: number) => Promise<Buffer>,
      clearCache?: (assetId: string) => Promise<void>,
    }
): BlobRepository {
  const mockBlobRepository: BlobRepository = {
    storeBlob: storeBlob || mock(() => Promise.resolve()),
    replaceBlob: replaceBlob || mock(() => Promise.resolve()),
    blobPath: blobPath || mock(() => ''),
    renameBlob: renameBlob || mock(() => Promise.resolve()),
    thumbnail: thumbnail || mock(() => Promise.resolve(Buffer.from(''))),
    clearCache: clearCache || mock(() => Promise.resolve()),
  };
  return mockBlobRepository;
}