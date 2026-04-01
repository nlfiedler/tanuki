//
// Copyright (c) 2025 Nathan Fiedler
//
import fs from 'node:fs/promises';
import path from 'node:path';
import type { GraphQLResolveInfo } from 'graphql';
import container from 'tanuki/server/container.ts';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { AssetInput } from 'tanuki/server/domain/entities/asset.ts';
import * as ops from 'tanuki/server/domain/entities/edit.ts';
import { Location } from 'tanuki/server/domain/entities/location.ts';
import {
  PendingParams,
  SearchParams,
  SortField,
  SortOrder,
  SearchResult
} from 'tanuki/server/domain/entities/search.ts';
import type {
  Asset as GQLAsset,
  AssetEdit,
  AssetInput as GQLAssetInput,
  AttributeCount,
  BrowseMeta,
  LocationValues,
  MutationEditArgs,
  MutationReplaceArgs,
  MutationUpdateArgs,
  Resolvers,
  QueryAssetArgs,
  QueryLookupArgs,
  QueryScanFetchByIdArgs,
  QueryScanFetchOffsetArgs,
  QuerySearchFetchByIdArgs,
  QuerySearchFetchOffsetArgs,
  QueryPendingArgs,
  QueryScanArgs,
  QuerySearchArgs,
  QueryTagsForAssetsArgs,
  SearchMeta,
  PendingParams as GQLPendingParams,
  SearchParams as GQLSearchParams
} from 'tanuki/generated/graphql.ts';
import logger from 'tanuki/server/logger.ts';

const settings: any = container.resolve('settingsRepository');
const blobs: any = container.resolve('blobRepository');

// The GraphQL schema
const schemaPath = path.join(import.meta.dirname, 'schema.graphql');
export const typeDefs = await fs.readFile(schemaPath, 'utf8');

export const resolvers: Resolvers = {
  Query: {
    async asset(
      _parent: any,
      args: QueryAssetArgs,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<GQLAsset> {
      logger.info('asset: %s', args.id);
      const getAsset: any = container.resolve('getAsset');
      const output = await getAsset(args.id);
      return assetToGQL(output);
    },

    async lookup(
      _parent: any,
      args: QueryLookupArgs,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<GQLAsset> {
      logger.info('lookup: %s', args.checksum);
      const getAssetByDigest: any = container.resolve('getAssetByDigest');
      const output = await getAssetByDigest(args.checksum);
      return assetToGQL(output);
    },

    async count(
      _parent: any,
      _args: any,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<number> {
      logger.info('count');
      const countAssets: any = container.resolve('countAssets');
      try {
        return countAssets();
      } catch (error: any) {
        logger.error('record count failed: %o', error);
      }
      return 0;
    },

    async tags(
      _parent: any,
      _args: any,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<AttributeCount[]> {
      logger.info('tags');
      const getTags: any = container.resolve('getTags');
      return getTags();
    },

    async locationParts(
      _parent: any,
      _args: any,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<AttributeCount[]> {
      logger.info('locationParts');
      const getLocationParts: any = container.resolve('getLocationParts');
      return getLocationParts();
    },

    async locationRecords(
      _parent: any,
      args: any,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<Location[]> {
      logger.info('locationRecords');
      const getLocationRecords: any = container.resolve('getLocationRecords');
      return getLocationRecords();
    },

    async locationValues(
      _parent: any,
      args: any,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<LocationValues> {
      logger.info('locationValues');
      const getLocationValues: any = container.resolve('getLocationValues');
      return getLocationValues();
    },

    async years(
      _parent: any,
      _args: any,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<AttributeCount[]> {
      logger.info('years');
      const getYears: any = container.resolve('getYears');
      return getYears();
    },

    async mediaTypes(
      _parent: any,
      _args: any,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<AttributeCount[]> {
      logger.info('mediaTypes');
      const getMediaTypes: any = container.resolve('getMediaTypes');
      return getMediaTypes();
    },

    async pending(
      _parent: any,
      args: QueryPendingArgs,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<SearchMeta> {
      logger.info('pending');
      const params = pendingParamsFromGQL(args.params);
      const findPending: any = container.resolve('findPending');
      const results = await findPending(params);
      logger.info('found %d pending assets', results.length);
      return paginateResults(results, args.offset, args.limit);
    },

    async search(
      _parent: any,
      args: QuerySearchArgs,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<SearchMeta> {
      const params = searchParamsFromGQL(args.params);
      const searchAssets: any = container.resolve('searchAssets');
      const results = await searchAssets(params);
      logger.info('search yielded %d results', results.length);
      return paginateResults(results, args.offset, args.limit);
    },

    async scan(
      _parent: any,
      args: QueryScanArgs,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<SearchMeta> {
      const query = args.query;
      const sortField = sortFieldFromGQL(args.sortField);
      const sortOrder = sortOrderFromGQL(args.sortOrder);
      const scanAssets: any = container.resolve('scanAssets');
      const results = await scanAssets(query, sortField, sortOrder);
      logger.info('scan yielded %d results', results.length);
      return paginateResults(results, args.offset, args.limit);
    },

    async tagsForAssets(
      _parent: any,
      args: QueryTagsForAssetsArgs,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<AttributeCount[]> {
      logger.info('tagsForAssets');
      const getAssetTags: any = container.resolve('getAssetTags');
      return getAssetTags(args.assets);
    },

    async scanFetchOffset(
      _parent: any,
      args: QueryScanFetchOffsetArgs,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<BrowseMeta> {
      const query = args.query;
      const sortField = sortFieldFromGQL(args.sortField);
      const sortOrder = sortOrderFromGQL(args.sortOrder);
      const scanAssets: any = container.resolve('scanAssets');
      const results = await scanAssets(query, sortField, sortOrder);
      logger.info('scanFetchOffset yielded %d results', results.length);
      return fetchResultOffset(results, args.offset);
    },

    async searchFetchOffset(
      _parent: any,
      args: QuerySearchFetchOffsetArgs,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<BrowseMeta> {
      const params = searchParamsFromGQL(args.params);
      const searchAssets: any = container.resolve('searchAssets');
      const results = await searchAssets(params);
      logger.info('searchFetchOffset yielded %d results', results.length);
      return fetchResultOffset(results, args.offset);
    },

    async scanFetchById(
      _parent: any,
      args: QueryScanFetchByIdArgs,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<BrowseMeta> {
      const query = args.query;
      const sortField = sortFieldFromGQL(args.sortField);
      const sortOrder = sortOrderFromGQL(args.sortOrder);
      const scanAssets: any = container.resolve('scanAssets');
      const results = await scanAssets(query, sortField, sortOrder);
      logger.info('scanFetchById yielded %d results', results.length);
      return fetchResultById(results, args.id);
    },

    async searchFetchById(
      _parent: any,
      args: QuerySearchFetchByIdArgs,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<BrowseMeta> {
      const params = searchParamsFromGQL(args.params);
      const searchAssets: any = container.resolve('searchAssets');
      const results = await searchAssets(params);
      logger.info('searchFetchById yielded %d results', results.length);
      return fetchResultById(results, args.id);
    }
  },

  Mutation: {
    async import(
      _parent: any,
      args: any,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<number> {
      logger.info('import');
      const uploadsPath = settings.get('UPLOAD_PATH');
      const importUploads: any = container.resolve('importUploads');
      return importUploads(uploadsPath);
    },

    async update(
      _parent: any,
      args: MutationUpdateArgs,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<GQLAsset> {
      logger.info('update');
      const updateAsset: any = container.resolve('updateAsset');
      const input = assetInputFromGQL(args.id, args.asset);
      const output = await updateAsset(input);
      return assetToGQL(output);
    },

    async edit(
      _parent: any,
      args: MutationEditArgs,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<number> {
      logger.info('edit');
      const editAssets: any = container.resolve('editAssets');
      const operations = operationsFromGQL(args.operations);
      return editAssets(args.assetIds, operations);
    },

    async replace(
      _parent: any,
      args: MutationReplaceArgs,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<GQLAsset> {
      logger.info('replace');
      const replaceAsset: any = container.resolve('replaceAsset');
      const output = await replaceAsset(args.oldAssetId, args.newAssetId);
      return assetToGQL(output);
    }
  }
};

function paginateResults(
  results: SearchResult[],
  offset?: number | null,
  limit?: number | null
): SearchMeta {
  const count = results.length;
  const off = boundedIntValue(offset, 0, 0, count);
  const lim = boundedIntValue(limit, 16, 1, 256);
  const pageRows = results.slice(off, off + lim).map((r) => {
    return Object.assign({}, r, {
      thumbnailUrl: blobs.thumbnailUrl(r.assetId, 960, 960),
      assetUrl: blobs.assetUrl(r.assetId)
    });
  });
  const lastPage = count == 0 ? 1 : Math.ceil(count / lim);
  return {
    results: pageRows,
    count,
    lastPage
  };
}

async function fetchResultOffset(
  results: SearchResult[],
  offset: number
): Promise<BrowseMeta> {
  if (results.length > offset) {
    const getAsset: any = container.resolve('getAsset');
    const asset = await getAsset(results[offset]!.assetId);
    return {
      asset: assetToGQL(asset),
      offset,
      lastOffset: results.length - 1
    };
  }
  return {
    asset: null,
    offset,
    lastOffset: results.length === 0 ? 0 : results.length - 1
  };
}

async function fetchResultById(
  results: SearchResult[],
  assetId: string
): Promise<BrowseMeta> {
  const getAsset: any = container.resolve('getAsset');
  const offset = results.findIndex((r) => r.assetId == assetId);
  const asset = await getAsset(assetId);
  return {
    asset: assetToGQL(asset),
    offset,
    lastOffset: results.length - 1
  };
}

function pendingParamsFromGQL(incoming: GQLPendingParams): PendingParams {
  const outgoing = new PendingParams();
  if (incoming.after) {
    // incoming date-time is expected to be a string
    outgoing.setAfterDate(new Date(incoming.after));
  }
  outgoing.setSortField(sortFieldFromGQL(incoming.sortField));
  outgoing.setSortOrder(sortOrderFromGQL(incoming.sortOrder));
  return outgoing;
}

function searchParamsFromGQL(incoming: GQLSearchParams): SearchParams {
  const outgoing = new SearchParams();
  if (incoming.mediaType) {
    outgoing.setMediaType(incoming.mediaType);
  }
  if (incoming.after) {
    // incoming date-time is expected to be a string
    outgoing.setAfterDate(new Date(incoming.after));
  }
  if (incoming.before) {
    // incoming date-time is expected to be a string
    outgoing.setBeforeDate(new Date(incoming.before));
  }
  if (Array.isArray(incoming.tags)) {
    outgoing.setTags(incoming.tags);
  }
  if (Array.isArray(incoming.locations)) {
    outgoing.setLocations(incoming.locations);
  }
  outgoing.setSortField(sortFieldFromGQL(incoming.sortField));
  outgoing.setSortOrder(sortOrderFromGQL(incoming.sortOrder));
  return outgoing;
}

function sortFieldFromGQL(field: string | null | undefined): SortField {
  if (field === 'DATE') {
    return SortField.Date;
  }
  if (field === 'FILENAME') {
    return SortField.Filename;
  }
  if (field === 'MEDIATYPE') {
    return SortField.MediaType;
  }
  return SortField.Identifier;
}

function sortOrderFromGQL(order: string | null | undefined): SortOrder {
  if (order === 'DESCENDING') {
    return SortOrder.Descending;
  }
  return SortOrder.Ascending;
}

/**
 * Constrain the given value to a range, using the fallback if necessary.
 *
 * @param value input value, possibly null or undefined.
 * @param fallback value to use if input value is not a number.
 * @param minimum minimum allowed value.
 * @param maximum maximum allowed value.
 * @returns value that is within the given range.
 */
function boundedIntValue(
  value: number | null | undefined,
  fallback: number,
  minimum: number,
  maximum: number
): number {
  const v: number = value ?? fallback;
  return Math.min(Math.max(Number.isNaN(v) ? fallback : v, minimum), maximum);
}

function assetInputFromGQL(
  assetId: string,
  incoming: GQLAssetInput
): AssetInput {
  const outgoing = new AssetInput(assetId);
  if (Array.isArray(incoming.tags)) {
    outgoing.setTags(incoming.tags);
  }
  if (incoming.caption) {
    outgoing.setCaption(incoming.caption);
  }
  if (incoming.location !== null) {
    // retain any blank fields on the input in order to allow clearing the
    // corresponding fields on the asset entity
    const location = Location.fromRaw(
      incoming.location?.label ?? null,
      incoming.location?.city ?? null,
      incoming.location?.region ?? null
    );
    outgoing.setLocation(location);
  }
  if (incoming.datetime) {
    // incoming date-time is expected to be a string
    outgoing.setDatetime(new Date(incoming.datetime));
  }
  if (incoming.mediaType) {
    outgoing.setMediaType(incoming.mediaType);
  }
  if (incoming.filename) {
    outgoing.setFilename(incoming.filename);
  }
  return outgoing;
}

function locationFieldFromGQL(
  field: string | null | undefined
): ops.LocationField {
  if (field === 'LABEL') {
    return ops.LocationField.Label;
  }
  if (field === 'CITY') {
    return ops.LocationField.City;
  }
  return ops.LocationField.Region;
}

function operationsFromGQL(edits: AssetEdit): ops.Operation[] {
  const operations: ops.Operation[] = [];
  if (edits.addTags) {
    for (const name of edits.addTags) {
      operations.push(new ops.TagAdd(name));
    }
  }
  if (edits.removeTags) {
    for (const name of edits.removeTags) {
      operations.push(new ops.TagRemove(name));
    }
  }
  if (edits.setLocation) {
    for (const le of edits.setLocation) {
      const field = locationFieldFromGQL(le.field);
      if (le.value) {
        operations.push(new ops.LocationSetField(field, le.value));
      } else {
        operations.push(new ops.LocationClearField(field));
      }
    }
  }
  if (edits.setDate) {
    operations.push(new ops.DatetimeSet(new Date(edits.setDate)));
  } else if (edits.addDays != 0) {
    // both adds and subtracts according to the sign of the value
    operations.push(new ops.DatetimeAddDays(edits.addDays));
  }
  return operations;
}

function assetToGQL(entity: Asset): GQLAsset {
  return {
    id: entity.key,
    checksum: entity.checksum,
    filename: entity.filename,
    filepath: entity.filepath(),
    byteLength: entity.byteLength,
    datetime: entity.bestDate(),
    mediaType: entity.mediaType,
    tags: entity.tags,
    caption: entity.caption,
    location: entity.location,
    assetUrl: blobs.assetUrl(entity.key)
  };
}
