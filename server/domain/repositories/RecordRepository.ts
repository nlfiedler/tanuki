//
// Copyright (c) 2025 Nathan Fiedler
//
import { Asset } from 'tanuki/server/domain/entities/Asset.ts';
import { AttributeCount } from '../entities/AttributeCount.ts';
import { Location } from 'tanuki/server/domain/entities/Location.ts';
import { SearchResult } from 'tanuki/server/domain/entities/SearchResult.ts';

/**
 * Repository for entity records.
 */
interface RecordRepository {

  /**
   * Return the number of asset records stored in the database.
   *
   * @returns number of asset records.
   */
  countAssets(): Promise<number>;

  /**
   * Retrieve an asset record by its unique identifier, throwing an error if not
   * found.
   *
   * @param assetId - unique asset identifier.
   * @returns asset entity or null if not found.
   */
  getAssetById(assetId: string): Promise<Asset | null>;

  /**
   * Retreive an asset record by its SHA-256 checksum, returning `null` if not
   * found.
   *
   * @param digest - hash digest of asset to be queried.
   * @returns asset entity or null if not found.
   */
  getAssetByDigest(digest: string): Promise<Asset | null>;

  /**
   * Return all of the tags and the number of assets associated with each.
   */
  allTags(): Promise<AttributeCount[]>;

  /**
   * Return all of the location parts and the number of assets associated with
   * each individual part (label separated from city separated from region).
   */
  allLocations(): Promise<AttributeCount[]>;

  /**
   * Return all of the location records (label, city, and region together).
   */
  rawLocations(): Promise<Location[]>;

  /**
   * Store the asset record in the database either as a new record or updating
   * an existing record, as determined by its unique identifier.
   *
   * @param asset - entity to be stored in the repository.
   */
  putAsset(asset: Asset): Promise<void>;

  /**
   * Search for assets that have all of the given tags.
   * 
   * @param tags - set of tags on which to query.
   * @returns list of search results.
   */
  queryByTags(tags: String[]): Promise<SearchResult[]>;

  /**
   * Search for assets whose location fields match all of the given values.
   *
   * For example, searching for `["paris","france"]` will return assets that
   * have both `"paris"` and `"france"` in the location column, such as in the
   * `city` and `region` fields.
   */
  queryByLocations(locations: string[]): Promise<SearchResult[]>;

  /**
   * Search for assets whose media type matches the one given.
   */
  queryByMediaType(media_type: string): Promise<SearchResult[]>;

  /**
   * Search for asssets whose best date is before the one given.
   */
  queryBeforeDate(before: Date): Promise<SearchResult[]>;

  /**
   * Search for asssets whose best date is equal to or after the one given.
   */
  queryAfterDate(after: Date): Promise<SearchResult[]>;

  /**
   * Search for assets whose best date is between the after and before dates.
   *
   * As with `queryAfterDate()`, the after value is inclusive.
   */
  queryDateRange(after: Date, before: Date): Promise<SearchResult[]>;

  /**
   * Query for assets that lack any tags, caption, and location that were
   * imported after a given date-time.
   */
  queryNewborn(after: Date): Promise<SearchResult[]>;
}

export { type RecordRepository };
