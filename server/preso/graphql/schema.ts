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
  LocationValues,
  MutationEditArgs,
  MutationUpdateArgs,
  Resolvers,
  QueryAssetArgs,
  QueryPendingArgs,
  QueryScanArgs,
  QuerySearchArgs,
  SearchMeta,
  PendingParams as GQLPendingParams,
  SearchParams as GQLSearchParams
} from 'tanuki/generated/graphql.ts';
import logger from 'tanuki/server/logger.ts';

const settings = container.resolve('settingsRepository');
const blobs = container.resolve('blobRepository');

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

    async locationValues(
      _parent: any,
      args: any,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<LocationValues> {
      const getLocationValues = container.resolve('getLocationValues');
      return await getLocationValues();
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
      const searchAssets = container.resolve('searchAssets');
      const results = await searchAssets(params);
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
      const scanAssets = container.resolve('scanAssets');
      const results = await scanAssets(query, sortField, sortOrder);
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
    },

    async edit(
      _parent: any,
      args: MutationEditArgs,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<number> {
      const editAssets = container.resolve('editAssets');
      const operations = operationsFromGQL(args.operations);
      return editAssets(args.assetIds, operations);
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

function operationsFromGQL(edits: AssetEdit[]): ops.Operation[] {
  const operations: ops.Operation[] = [];
  for (const edit of edits) {
    if (edit.addTags) {
      for (const name of edit.addTags) {
        operations.push(new ops.TagAdd(name));
      }
    }
    if (edit.removeTags) {
      for (const name of edit.removeTags) {
        operations.push(new ops.TagRemove(name));
      }
    }
    if (edit.setLocation) {
      for (const le of edit.setLocation) {
        const field = locationFieldFromGQL(le.field);
        if (le.value) {
          operations.push(new ops.LocationSetField(field, le.value));
        } else {
          operations.push(new ops.LocationClearField(field));
        }
      }
    }
    if (edit.setDate) {
      operations.push(new ops.DatetimeSet(new Date(edit.setDate)));
    } else if (edit.addDays) {
      operations.push(new ops.DatetimeAddDays(edit.addDays));
    } else if (edit.subtractDays) {
      operations.push(new ops.DatetimeSubDays(edit.subtractDays));
    }
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
