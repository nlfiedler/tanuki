//
// Copyright (c) 2025 Nathan Fiedler
//
import fs from 'node:fs/promises';
import path from 'node:path';
import type { GraphQLResolveInfo } from 'graphql';
import container from 'tanuki/server/container.ts';
import { Asset } from 'tanuki/server/domain/entities/Asset.ts';
import { AssetInput } from 'tanuki/server/domain/entities/AssetInput.ts';
import { Location } from 'tanuki/server/domain/entities/Location.ts';
import {
  PendingParams,
  SearchParams,
  SortField,
  SortOrder
} from 'tanuki/server/domain/entities/SearchParams.ts';
import { SearchResult } from 'tanuki/server/domain/entities/SearchResult.ts';
import type {
  Asset as GQLAsset,
  AssetInput as GQLAssetInput,
  MutationUpdateArgs,
  Resolvers,
  QueryPendingArgs,
  QuerySearchArgs,
  SearchMeta,
  PendingParams as GQLPendingParams,
  SearchParams as GQLSearchParams
} from 'tanuki/generated/graphql.ts';

// The GraphQL schema
const schemaPath = path.join('public', 'schema.graphql');
export const typeDefs = await fs.readFile(schemaPath, 'utf8');

export const resolvers: Resolvers = {
  Query: {
    async count(_parent: any, _args: any, _context: any, _info: GraphQLResolveInfo) {
      const countAssets = container.resolve('countAssets');
      return countAssets();
    },

    async tags(_parent: any, _args: any, _context: any, _info: GraphQLResolveInfo) {
      const getTags = container.resolve('getTags');
      return getTags();
    },

    async locationParts(_parent: any, _args: any, _context: any, _info: GraphQLResolveInfo) {
      const getLocationParts = container.resolve('getLocationParts');
      return getLocationParts();
    },

    async locationRecords(_parent: any, args: any, _context: any, _info: GraphQLResolveInfo): Promise<Location[]> {
      const getLocationRecords = container.resolve('getLocationRecords');
      return await getLocationRecords();
    },

    async pending(_parent: any, args: QueryPendingArgs, _context: any, _info: GraphQLResolveInfo): Promise<SearchMeta> {
      const params = pendingParamsFromGQL(args.params);
      const findPending = container.resolve('findPending');
      const results = await findPending(params);
      return paginateResults(results, args.offset, args.limit);
    },

    async search(_parent: any, args: QuerySearchArgs, _context: any, _info: GraphQLResolveInfo): Promise<SearchMeta> {
      const params = searchParamsFromGQL(args.params);
      const searchAssets = container.resolve('searchAssets');
      const results = await searchAssets(params);
      return paginateResults(results, args.offset, args.limit);
    },
  },

  Mutation: {
    async update(_parent: any, args: MutationUpdateArgs, _context: any, _info: GraphQLResolveInfo): Promise<GQLAsset> {
      const updateAsset = container.resolve('updateAsset');
      const input = assetInputFromGQL(args.id, args.asset);
      const output = await updateAsset(input);
      return assetToGQL(output);
    }
  }
};

function paginateResults(results: SearchResult[], offset?: number | null, limit?: number | null): SearchMeta {
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
  const v: number = value ? value : fallback;
  return Math.min(Math.max(isNaN(v) ? fallback : v, minimum), maximum);
}

function assetInputFromGQL(assetId: string, incoming: GQLAssetInput): AssetInput {
  const outgoing = new AssetInput(assetId);
  if (Array.isArray(incoming.tags)) {
    outgoing.setTags(incoming.tags);
  }
  if (incoming.caption) {
    outgoing.setCaption(incoming.caption);
  }
  if (incoming.location !== null) {
    const location = Location.fromRaw(
      incoming.location?.label || null,
      incoming.location?.city || null,
      incoming.location?.region || null
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
    byteLength: entity.byteLength,
    datetime: entity.bestDate(),
    mediaType: entity.mediaType,
    tags: entity.tags,
    caption: entity.caption,
    location: entity.location,
  };
}
