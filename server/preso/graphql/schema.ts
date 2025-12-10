//
// Copyright (c) 2025 Nathan Fiedler
//
import fs from 'node:fs/promises';
import path from 'node:path';
import type { GraphQLResolveInfo } from 'graphql';
import container from 'tanuki/server/container.ts';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { AssetInput } from 'tanuki/server/domain/entities/asset.ts';
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
  AssetInput as GQLAssetInput,
  MutationUpdateArgs,
  Resolvers,
  QueryAssetArgs,
  QueryPendingArgs,
  QuerySearchArgs,
  SearchMeta,
  PendingParams as GQLPendingParams,
  SearchParams as GQLSearchParams
} from 'tanuki/generated/graphql.ts';
import logger from 'tanuki/server/logger.ts';

const settings = container.resolve('settingsRepository');

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
      const getAsset = container.resolve('getAsset');
      const output = await getAsset(args.id);
      return assetToGQL(output);
    },

    async count(
      _parent: any,
      _args: any,
      _context: any,
      _info: GraphQLResolveInfo
    ) {
      const countAssets = container.resolve('countAssets');
      try {
        return countAssets();
      } catch (error: any) {
        logger.error('record count failed: %o', error);
      }
    },

    async tags(
      _parent: any,
      _args: any,
      _context: any,
      _info: GraphQLResolveInfo
    ) {
      const getTags = container.resolve('getTags');
      return getTags();
    },

    async locationParts(
      _parent: any,
      _args: any,
      _context: any,
      _info: GraphQLResolveInfo
    ) {
      const getLocationParts = container.resolve('getLocationParts');
      return getLocationParts();
    },

    async locationRecords(
      _parent: any,
      args: any,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<Location[]> {
      const getLocationRecords = container.resolve('getLocationRecords');
      return await getLocationRecords();
    },

    async years(
      _parent: any,
      _args: any,
      _context: any,
      _info: GraphQLResolveInfo
    ) {
      const getYears = container.resolve('getYears');
      return getYears();
    },

    async mediaTypes(
      _parent: any,
      _args: any,
      _context: any,
      _info: GraphQLResolveInfo
    ) {
      const getMediaTypes = container.resolve('getMediaTypes');
      return getMediaTypes();
    },

    async pending(
      _parent: any,
      args: QueryPendingArgs,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<SearchMeta> {
      const params = pendingParamsFromGQL(args.params);
      const findPending = container.resolve('findPending');
      const results = await findPending(params);
      return paginateResults(results, args.offset, args.limit);
    },

    async search(
      _parent: any,
      args: QuerySearchArgs,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<SearchMeta> {
      const params = searchParamsFromGQL(args.params);
      const searchAssets = container.resolve('searchAssets');
      const results = await searchAssets(params);
      return paginateResults(results, args.offset, args.limit);
    }
  },

  Mutation: {
    async import(
      _parent: any,
      args: any,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<number> {
      const uploadsPath = settings.get('UPLOAD_PATH');
      const importUploads = container.resolve('importUploads');
      return await importUploads(uploadsPath);
    },

    async update(
      _parent: any,
      args: MutationUpdateArgs,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<GQLAsset> {
      const updateAsset = container.resolve('updateAsset');
      const input = assetInputFromGQL(args.id, args.asset);
      const output = await updateAsset(input);
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
  const pageRows = results.slice(off, off + lim);
  const lastPage = count == 0 ? 1 : Math.ceil(count / lim);
  return {
    results: pageRows,
    count,
    lastPage
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
    location: entity.location
  };
}
