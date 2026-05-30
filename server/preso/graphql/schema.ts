//
// Copyright (c) 2025 Nathan Fiedler
//
import fs from 'node:fs/promises';
import path from 'node:path';
import type { GraphQLResolveInfo } from 'graphql';
import container from 'tanuki/server/container.ts';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { AssetInput } from 'tanuki/server/domain/entities/asset.ts';
import { AssetMetadata } from 'tanuki/server/domain/entities/asset-metadata.ts';
import {
  SyntheticData,
  SyntheticStatus
} from 'tanuki/server/domain/entities/synthetic-data.ts';
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
  AssetMetadata as GQLAssetMetadata,
  SyntheticData as GQLSyntheticData,
  SyntheticStatus as GQLSyntheticStatus,
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
import type { GraphQLContext } from 'tanuki/server/preso/graphql/metadata-loader.ts';
import logger from 'tanuki/server/logger.ts';

const settings: any = container.resolve('settingsRepository');
const blobs: any = container.resolve('blobRepository');

// The GraphQL schema
const schemaPath = path.join(import.meta.dirname, 'schema.graphql');
export const typeDefs = await fs.readFile(schemaPath, 'utf8');

export const resolvers: Resolvers = {
  Asset: {
    metadata: async (parent: any, _args, context: GraphQLContext) => {
      // The asset resolver attaches `metadata` to the parent when the
      // underlying read path already hydrated it (e.g. getAssetById). In that
      // case we hand back the inline value to avoid an extra fetch. For paths
      // where metadata is not on the parent, fall back to the DataLoader.
      if (parent?.metadata !== undefined) {
        return metadataToGQL(parent.metadata);
      }
      const meta = await context.metadataLoader.load(parent.id);
      return metadataToGQL(meta);
    },
    synthetic: async (parent: any, _args, context: GraphQLContext) => {
      const data =
        parent?.synthetic === undefined
          ? await context.syntheticLoader.load(parent.id)
          : parent.synthetic;
      return syntheticToGQL(data, parent.id);
    },
    syntheticStatus: async (parent: any, _args, context: GraphQLContext) => {
      // Always go through the loader: the surfaced status is the worse of the
      // labels status (on the record, carried inline) and the faces status
      // (in the face store), so the inline value alone is not authoritative.
      const status = await context.syntheticStatusLoader.load(parent.id);
      return statusToGQL(status);
    }
  },
  SearchResult: {
    metadata: async (parent: any, _args, context: GraphQLContext) => {
      const meta = await context.metadataLoader.load(parent.assetId);
      return metadataToGQL(meta);
    },
    synthetic: async (parent: any, _args, context: GraphQLContext) => {
      const data = await context.syntheticLoader.load(parent.assetId);
      return syntheticToGQL(data, parent.assetId);
    },
    syntheticStatus: async (parent: any, _args, context: GraphQLContext) => {
      const status = await context.syntheticStatusLoader.load(parent.assetId);
      return statusToGQL(status);
    },
    previewUrl: (
      parent: any,
      args: { width?: number | null; height?: number | null }
    ) => {
      const w = args.width ?? null;
      const h = args.height ?? null;
      if (w !== null && w > 0) {
        return blobs.previewUrl(parent.assetId, { width: w });
      }
      if (h !== null && h > 0) {
        return blobs.previewUrl(parent.assetId, { height: h });
      }
      return blobs.previewUrl(parent.assetId, { height: 800 });
    }
  },
  SyntheticData: {
    // People are resolved from the face store via a per-request DataLoader,
    // keyed by the asset id threaded onto the parent by the `synthetic` field.
    people: async (parent: any, _args, context: GraphQLContext) => {
      const summaries = await context.peopleLoader.load(parent._assetId);
      return summaries.map((s) => personToGQL(s));
    }
  },
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

    async labels(
      _parent: any,
      _args: any,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<any[]> {
      logger.info('labels');
      const getLabels: any = container.resolve('getLabels');
      const entries = await getLabels();
      // map domain LabelEntry -> GraphQL LabelEntry (build the thumbnail URL)
      return entries.map((e: any) => ({
        label: e.label,
        count: e.count,
        thumbnail: blobs.thumbnailUrl(e.thumbnailAssetId, 240, 240)
      }));
    },

    async assetsByLabel(
      _parent: any,
      args: { label: string; offset?: number | null; limit?: number | null },
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<SearchMeta> {
      logger.info('assetsByLabel');
      const assetsByLabel: any = container.resolve('assetsByLabel');
      const results = await assetsByLabel(args.label);
      return paginateResults(results, args.offset, args.limit);
    },

    async people(
      _parent: any,
      args: { includeHidden?: boolean | null },
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<any[]> {
      logger.info('people');
      const getPeople: any = container.resolve('getPeople');
      const summaries = await getPeople(args.includeHidden ?? false);
      return summaries.map((s: any) => personToGQL(s));
    },

    async personFaces(
      _parent: any,
      args: { id: string },
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<any[]> {
      logger.info('personFaces');
      const getPersonFaces: any = container.resolve('getPersonFaces');
      const faces = await getPersonFaces(args.id);
      return faces.map((f: any) => ({
        id: f.id,
        assetId: f.assetId,
        thumbnail: faceThumbUrl(f.id)
      }));
    },

    async assetsByPerson(
      _parent: any,
      args: { id: string; offset?: number | null; limit?: number | null },
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<SearchMeta> {
      logger.info('assetsByPerson');
      const assetsByPerson: any = container.resolve('assetsByPerson');
      // The face store pages at the source, so pass offset/limit through and
      // build the meta from the store's total rather than slicing a full list.
      const offset = Math.max(0, args.offset ?? 0);
      const limit = boundedIntValue(args.limit, 16, 1, 256);
      const { results, total } = await assetsByPerson(args.id, offset, limit);
      const lastPage = total === 0 ? 1 : Math.ceil(total / limit);
      return {
        results: shapeSearchRows(results),
        count: total,
        lastPage
      };
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
    },

    async updateVideoDates(
      _parent: any,
      _args: any,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<number> {
      logger.info('updateVideoDates');
      const updateVideoDates: any = container.resolve('updateVideoDates');
      return updateVideoDates();
    },

    async backfillImageMetadata(
      _parent: any,
      _args: any,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<number> {
      logger.info('backfillImageMetadata');
      const backfill: any = container.resolve('backfillImageMetadata');
      return backfill();
    },

    async backfillVideoMetadata(
      _parent: any,
      _args: any,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<number> {
      logger.info('backfillVideoMetadata');
      const backfill: any = container.resolve('backfillVideoMetadata');
      return backfill();
    },

    async fixOriginalDates(
      _parent: any,
      _args: any,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<number> {
      logger.info('fixOriginalDates');
      const fix: any = container.resolve('fixOriginalDates');
      return fix();
    },

    async backfillLabels(
      _parent: any,
      _args: any,
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<number> {
      logger.info('backfillLabels');
      const backfill: any = container.resolve('backfillLabels');
      return backfill();
    },

    async retrySyntheticJobs(
      _parent: any,
      args: { kind?: string | null },
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<number> {
      logger.info('retrySyntheticJobs');
      const retry: any = container.resolve('retrySyntheticJobs');
      return retry(args.kind);
    },

    async backfillFaceRecognition(
      _parent: any,
      args: { force?: boolean | null },
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<number> {
      logger.info('backfillFaceRecognition');
      const backfill: any = container.resolve('backfillFaceRecognition');
      return backfill(args.force ?? false);
    },

    async renamePerson(
      _parent: any,
      args: { id: string; name?: string | null },
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<any> {
      logger.info('renamePerson');
      const renamePerson: any = container.resolve('renamePerson');
      return personOrThrow(await renamePerson(args.id, args.name ?? null), args.id);
    },

    async mergePeople(
      _parent: any,
      args: { sourceId: string; targetId: string },
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<any> {
      logger.info('mergePeople');
      const mergePeople: any = container.resolve('mergePeople');
      return personOrThrow(
        await mergePeople(args.sourceId, args.targetId),
        args.targetId
      );
    },

    async reassignFaces(
      _parent: any,
      args: { faceIds: string[]; personId?: string | null },
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<any> {
      logger.info('reassignFaces');
      const reassignFaces: any = container.resolve('reassignFaces');
      // reassignFaces always resolves to a destination person (it creates one
      // when personId is null), so a null summary is a genuine internal error.
      const summary = await reassignFaces(args.faceIds, args.personId ?? null);
      if (!summary) throw new Error('reassignFaces produced no person');
      return personToGQL(summary);
    },

    async hidePerson(
      _parent: any,
      args: { id: string; hidden: boolean },
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<any> {
      logger.info('hidePerson');
      const hidePerson: any = container.resolve('hidePerson');
      return personOrThrow(await hidePerson(args.id, args.hidden), args.id);
    },

    async setPersonThumbnail(
      _parent: any,
      args: { id: string; faceId: string },
      _context: any,
      _info: GraphQLResolveInfo
    ): Promise<any> {
      logger.info('setPersonThumbnail');
      const setPersonThumbnail: any = container.resolve('setPersonThumbnail');
      return personOrThrow(
        await setPersonThumbnail(args.id, args.faceId),
        args.id
      );
    }
  }
};

/**
 * Attach the per-row URL fields a SearchMeta row needs. `previewUrl` is
 * computed lazily by the SearchResult field resolver, so it is omitted here;
 * the cast reconciles the shaped rows with SearchMeta's row type.
 */
function shapeSearchRows(results: SearchResult[]): SearchMeta['results'] {
  return results.map((r) =>
    Object.assign({}, r, {
      thumbnailUrl: blobs.thumbnailUrl(r.assetId, 960, 960),
      assetUrl: blobs.assetUrl(r.assetId)
    })
  ) as unknown as SearchMeta['results'];
}

function paginateResults(
  results: SearchResult[],
  offset?: number | null,
  limit?: number | null
): SearchMeta {
  const count = results.length;
  const off = boundedIntValue(offset, 0, 0, count);
  const lim = boundedIntValue(limit, 16, 1, 256);
  const lastPage = count == 0 ? 1 : Math.ceil(count / lim);
  return {
    results: shapeSearchRows(results.slice(off, off + lim)),
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
    assetUrl: blobs.assetUrl(entity.key),
    // pass the AssetMetadata and SyntheticData instances through verbatim;
    // the field resolvers coerce them on demand (avoids double-coercion).
    metadata: entity.metadata as unknown as GQLAssetMetadata | null,
    synthetic: entity.synthetic as unknown as GQLSyntheticData | null,
    syntheticStatus: statusToGQL(entity.syntheticStatus)
  };
}

/**
 * Shape a `SyntheticData` for GraphQL. Always returns an object (never null) so
 * the `people` field is reachable even when an asset produced no labels; the
 * asset id is threaded on (as `_assetId`, not part of the schema) for the
 * `people` field resolver's DataLoader.
 */
function syntheticToGQL(
  synthetic: SyntheticData | null | undefined,
  assetId: string
): GQLSyntheticData {
  return {
    labels: synthetic?.labels ?? [],
    primaryLabel: synthetic?.primaryLabel ?? null,
    _assetId: assetId
  } as unknown as GQLSyntheticData;
}

/** URL for a face crop thumbnail (served by the /faces route). */
function faceThumbUrl(faceId: string | null): string {
  // Persons always have at least one face, but fall back defensively.
  return faceId ? `/faces/${faceId}/thumb` : '/placeholder.svg';
}

/** Map a domain PersonSummary to the GraphQL Person shape. */
function personToGQL(summary: any): any {
  return {
    id: summary.person.id,
    name: summary.person.name,
    thumbnail: faceThumbUrl(summary.representativeFaceId),
    hidden: summary.person.hidden,
    faceCount: summary.faceCount
  };
}

/** Coerce a possibly-null person summary into a GQL person, or throw. */
function personOrThrow(summary: any, id: string): any {
  if (!summary) throw new Error(`no such person: ${id}`);
  return personToGQL(summary);
}

/**
 * Convert a domain SyntheticStatus to the codegen'd GraphQL enum. TypeScript
 * treats the two enum declarations as nominally distinct even though their
 * string values match exactly, so this helper centralizes the (safe) coercion.
 */
function statusToGQL(status: SyntheticStatus): GQLSyntheticStatus {
  return status as unknown as GQLSyntheticStatus;
}

function metadataToGQL(
  metadata: AssetMetadata | null | undefined
): GQLAssetMetadata | null {
  if (!metadata) return null;
  return {
    cameraMake: metadata.cameraMake,
    cameraModel: metadata.cameraModel,
    lensMake: metadata.lensMake,
    lensModel: metadata.lensModel,
    exposureTime: metadata.exposureTime,
    fNumber: metadata.fNumber,
    iso: metadata.iso,
    focalLength35mm: metadata.focalLength35mm,
    originalDateOffset: metadata.originalDateOffset,
    gpsLatitude: metadata.gpsLatitude,
    gpsLongitude: metadata.gpsLongitude,
    displayWidth: metadata.displayWidth,
    displayHeight: metadata.displayHeight,
    duration: metadata.duration,
    frameRate: metadata.frameRate,
    videoCodec: metadata.videoCodec,
    byteLength: metadata.byteLength
  };
}
