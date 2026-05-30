//
// Copyright (c) 2025 Nathan Fiedler
//
import { mock } from 'bun:test';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { AssetMetadata } from 'tanuki/server/domain/entities/asset-metadata.ts';
import { AttributeCount } from 'tanuki/server/domain/entities/attributes.ts';
import {
  SyntheticData,
  SyntheticStatus
} from 'tanuki/server/domain/entities/synthetic-data.ts';
import {
  Coordinates,
  Geocoded,
  Location
} from 'tanuki/server/domain/entities/location.ts';
import { SearchResult } from 'tanuki/server/domain/entities/search.ts';
import {
  Person,
  type Face,
  type JobKind,
  type PersonSummary,
  type SyntheticJob
} from 'tanuki/server/domain/entities/face.ts';
import { type BlobRepository } from 'tanuki/server/domain/repositories/blob-repository.ts';
import { type FaceStore } from 'tanuki/server/domain/repositories/face-store.ts';
import { type LocationRepository } from 'tanuki/server/domain/repositories/location-repository.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';
import { type SearchRepository } from 'tanuki/server/domain/repositories/search-repository.ts';

/* eslint-disable unicorn/no-useless-undefined */

/**
 * Helper for producing a mock record repository implementation. Any undefined
 * functions will return null, empty string, zero, etc.
 */
export function recordRepositoryMock({
  countAssets = undefined,
  getAssetById = undefined,
  getAssetByDigest = undefined,
  allTags = undefined,
  allLocations = undefined,
  rawLocations = undefined,
  allYears = undefined,
  allMediaTypes = undefined,
  allPrimaryLabels = undefined,
  queryByLabel = undefined,
  latestAssetByLabel = undefined,
  putAsset = undefined,
  deleteAsset = undefined,
  queryByTags = undefined,
  queryByLocations = undefined,
  queryByMediaType = undefined,
  queryBeforeDate = undefined,
  queryAfterDate = undefined,
  queryDateRange = undefined,
  queryNewborn = undefined,
  fetchAssets = undefined,
  storeAssets = undefined,
  fetchMetadata = undefined,
  fetchSynthetic = undefined,
  fetchSyntheticStatus = undefined,
  setSynthetic = undefined
}: {
  countAssets?: () => Promise<number>;
  getAssetById?: (assetId: string) => Promise<Asset | null>;
  getAssetByDigest?: (digest: string) => Promise<Asset | null>;
  allTags?: () => Promise<AttributeCount[]>;
  allLocations?: () => Promise<AttributeCount[]>;
  rawLocations?: () => Promise<Location[]>;
  allYears?: () => Promise<AttributeCount[]>;
  allMediaTypes?: () => Promise<AttributeCount[]>;
  allPrimaryLabels?: () => Promise<AttributeCount[]>;
  queryByLabel?: (label: string) => Promise<SearchResult[]>;
  latestAssetByLabel?: (
    label: string
  ) => Promise<{ assetId: string; primaryLabel: string } | null>;
  putAsset?: (asset: Asset) => Promise<void>;
  deleteAsset?: (assetId: string) => Promise<void>;
  queryByTags?: (tags: string[]) => Promise<SearchResult[]>;
  queryByLocations?: (locations: string[]) => Promise<SearchResult[]>;
  queryByMediaType?: (media_type: string) => Promise<SearchResult[]>;
  queryBeforeDate?: (before: Date) => Promise<SearchResult[]>;
  queryAfterDate?: (after: Date) => Promise<SearchResult[]>;
  queryDateRange?: (after: Date, before: Date) => Promise<SearchResult[]>;
  queryNewborn?: (after: Date) => Promise<SearchResult[]>;
  fetchAssets?: (cursor: any, limit: number) => Promise<[Asset[], any]>;
  storeAssets?: (incoming: Asset[]) => Promise<void>;
  fetchMetadata?: (
    assetIds: string[]
  ) => Promise<Map<string, AssetMetadata | null>>;
  fetchSynthetic?: (
    assetIds: string[]
  ) => Promise<Map<string, SyntheticData | null>>;
  fetchSyntheticStatus?: (
    assetIds: string[]
  ) => Promise<Map<string, SyntheticStatus>>;
  setSynthetic?: (
    assetId: string,
    data: SyntheticData | null,
    status: SyntheticStatus
  ) => Promise<void>;
}): RecordRepository {
  const mockRecordRepository: RecordRepository = {
    countAssets: countAssets || mock(() => Promise.resolve(0)),
    getAssetById: getAssetById || mock(() => Promise.resolve(null)),
    getAssetByDigest: getAssetByDigest || mock(() => Promise.resolve(null)),
    allTags: allTags || mock(() => Promise.resolve([])),
    allLocations: allLocations || mock(() => Promise.resolve([])),
    rawLocations: rawLocations || mock(() => Promise.resolve([])),
    allYears: allYears || mock(() => Promise.resolve([])),
    allMediaTypes: allMediaTypes || mock(() => Promise.resolve([])),
    allPrimaryLabels: allPrimaryLabels || mock(() => Promise.resolve([])),
    queryByLabel: queryByLabel || mock(() => Promise.resolve([])),
    latestAssetByLabel:
      latestAssetByLabel || mock(() => Promise.resolve(null)),
    putAsset: putAsset || mock(() => Promise.resolve()),
    deleteAsset: deleteAsset || mock(() => Promise.resolve()),
    queryByTags: queryByTags || mock(() => Promise.resolve([])),
    queryByLocations: queryByLocations || mock(() => Promise.resolve([])),
    queryByMediaType: queryByMediaType || mock(() => Promise.resolve([])),
    queryBeforeDate: queryBeforeDate || mock(() => Promise.resolve([])),
    queryAfterDate: queryAfterDate || mock(() => Promise.resolve([])),
    queryDateRange: queryDateRange || mock(() => Promise.resolve([])),
    queryNewborn: queryNewborn || mock(() => Promise.resolve([])),
    fetchAssets:
      fetchAssets ||
      mock(() =>
        Promise.resolve([new Array<Asset>(), 'done'] as [Asset[], any])
      ),
    storeAssets: storeAssets || mock(() => Promise.resolve()),
    fetchMetadata:
      fetchMetadata ||
      mock(() => Promise.resolve(new Map<string, AssetMetadata | null>())),
    fetchSynthetic:
      fetchSynthetic ||
      mock(() => Promise.resolve(new Map<string, SyntheticData | null>())),
    fetchSyntheticStatus:
      fetchSyntheticStatus ||
      mock(() => Promise.resolve(new Map<string, SyntheticStatus>())),
    setSynthetic: setSynthetic || mock(() => Promise.resolve())
  };
  return mockRecordRepository;
}

/**
 * Helper for producing a mock blob repository implementation. Any undefined
 * functions will return null, empty string, zero, etc.
 */
export function blobRepositoryMock({
  storeBlob = undefined,
  deleteBlob = undefined,
  fetchRange = undefined,
  assetUrl = undefined,
  thumbnailUrl = undefined,
  previewUrl = undefined,
  fetchMetadata = undefined
}: {
  storeBlob?: (filepath: string, asset: Asset) => Promise<void>;
  deleteBlob?: (assetId: string) => Promise<void>;
  fetchRange?: (
    assetId: string,
    start: number,
    end: number
  ) => Promise<Buffer>;
  assetUrl?: (assetId: string) => string;
  thumbnailUrl?: (assetId: string, width: number, height: number) => string;
  previewUrl?: (
    assetId: string,
    opts: { width: number } | { height: number }
  ) => string;
  fetchMetadata?: (
    assetId: string,
    mediaType: string
  ) => Promise<object | null>;
}): BlobRepository {
  const mockBlobRepository: BlobRepository = {
    storeBlob: storeBlob || mock(() => Promise.resolve()),
    deleteBlob: deleteBlob || mock(() => Promise.resolve()),
    fetchRange: fetchRange || mock(() => Promise.resolve(Buffer.alloc(0))),
    assetUrl: assetUrl || mock(() => ''),
    thumbnailUrl: thumbnailUrl || mock(() => ''),
    previewUrl: previewUrl || mock(() => ''),
    fetchMetadata: fetchMetadata || mock(() => Promise.resolve(null))
  };
  return mockBlobRepository;
}

/**
 * Helper for producing a mock location repository implementation. Any undefined
 * functions will return null, empty string, zero, etc.
 */
export function locationRepositoryMock({
  findLocation = undefined
}: {
  findLocation?: (coords: Coordinates) => Promise<Geocoded | null>;
}): LocationRepository {
  const mockLocationRepository: LocationRepository = {
    findLocation: findLocation || mock(() => Promise.resolve(null))
  };
  return mockLocationRepository;
}

/**
 * Helper for producing a mock search repository implementation. Any undefined
 * functions will return null, empty string, zero, etc.
 */
export function searchRepositoryMock({
  put = undefined,
  get = undefined,
  clear = undefined
}: {
  put?: (key: string, results: SearchResult[]) => Promise<void>;
  get?: (key: string) => Promise<SearchResult[] | undefined>;
  clear?: () => Promise<void>;
}): SearchRepository {
  const mockSearchRepository: SearchRepository = {
    put: put || mock(() => Promise.resolve()),
    get: get || mock(() => Promise.resolve(undefined)),
    clear: clear || mock(() => Promise.resolve())
  };
  return mockSearchRepository;
}

/**
 * Helper for producing a mock face store implementation. Any undefined
 * functions return a benign empty result (no job, empty maps, etc.).
 */
export function faceStoreMock({
  enqueueJob = undefined,
  claimNextJob = undefined,
  requeueJob = undefined,
  pendingJobCount = undefined,
  hasPendingJob = undefined,
  fetchPeopleByAssetIds = undefined,
  assetIdsByPerson = undefined,
  deleteByAssetId = undefined,
  insertFace = undefined,
  nearestPerson = undefined,
  createPerson = undefined,
  listPeople = undefined,
  getPersonSummary = undefined,
  personIdsByName = undefined,
  facesForPerson = undefined,
  faceThumbnail = undefined,
  renamePerson = undefined,
  mergePeople = undefined,
  reassignFaces = undefined,
  hidePerson = undefined,
  setPersonThumbnail = undefined,
  allFaceAssetIds = undefined,
  setFacesStatus = undefined,
  fetchFacesStatus = undefined,
  assetIdsWithFacesStatus = undefined,
  modelVersionsByAssets = undefined
}: {
  enqueueJob?: (
    assetId: string,
    kind: JobKind,
    priority?: number
  ) => Promise<number>;
  claimNextJob?: () => Promise<SyntheticJob | null>;
  requeueJob?: (job: SyntheticJob, error: string) => Promise<number>;
  pendingJobCount?: (kind?: JobKind) => Promise<number>;
  hasPendingJob?: (assetId: string, kind: JobKind) => Promise<boolean>;
  fetchPeopleByAssetIds?: (
    assetIds: string[]
  ) => Promise<Map<string, PersonSummary[]>>;
  assetIdsByPerson?: (
    personId: string,
    offset: number,
    limit: number
  ) => Promise<{ ids: string[]; total: number }>;
  deleteByAssetId?: (assetId: string) => Promise<void>;
  insertFace?: (face: Face) => Promise<void>;
  nearestPerson?: (
    embedding: Float32Array,
    modelVersion: string
  ) => Promise<{ personId: string; score: number } | null>;
  createPerson?: () => Promise<Person>;
  listPeople?: (includeHidden: boolean) => Promise<PersonSummary[]>;
  getPersonSummary?: (id: string) => Promise<PersonSummary | null>;
  personIdsByName?: (name: string) => Promise<string[]>;
  facesForPerson?: (personId: string) => Promise<Face[]>;
  faceThumbnail?: (faceId: string) => Promise<Uint8Array | null>;
  renamePerson?: (id: string, name: string | null) => Promise<void>;
  mergePeople?: (sourceId: string, targetId: string) => Promise<void>;
  reassignFaces?: (
    faceIds: string[],
    personId: string | null
  ) => Promise<string>;
  hidePerson?: (id: string, hidden: boolean) => Promise<void>;
  setPersonThumbnail?: (id: string, faceId: string) => Promise<void>;
  allFaceAssetIds?: () => Promise<string[]>;
  setFacesStatus?: (
    assetId: string,
    status: SyntheticStatus
  ) => Promise<void>;
  fetchFacesStatus?: (
    assetIds: string[]
  ) => Promise<Map<string, SyntheticStatus>>;
  assetIdsWithFacesStatus?: (status: SyntheticStatus) => Promise<string[]>;
  modelVersionsByAssets?: (
    assetIds: string[]
  ) => Promise<Map<string, Set<string>>>;
}): FaceStore {
  const mockFaceStore: FaceStore = {
    enqueueJob: enqueueJob || mock(() => Promise.resolve(1)),
    claimNextJob: claimNextJob || mock(() => Promise.resolve(null)),
    requeueJob: requeueJob || mock(() => Promise.resolve(1)),
    pendingJobCount: pendingJobCount || mock(() => Promise.resolve(0)),
    hasPendingJob: hasPendingJob || mock(() => Promise.resolve(false)),
    fetchPeopleByAssetIds:
      fetchPeopleByAssetIds ||
      mock(() => Promise.resolve(new Map<string, PersonSummary[]>())),
    assetIdsByPerson:
      assetIdsByPerson ||
      mock(() => Promise.resolve({ ids: [] as string[], total: 0 })),
    deleteByAssetId: deleteByAssetId || mock(() => Promise.resolve()),
    insertFace: insertFace || mock(() => Promise.resolve()),
    nearestPerson: nearestPerson || mock(() => Promise.resolve(null)),
    createPerson:
      createPerson ||
      mock(() => Promise.resolve(new Person('mock-person'))),
    listPeople: listPeople || mock(() => Promise.resolve([] as PersonSummary[])),
    getPersonSummary: getPersonSummary || mock(() => Promise.resolve(null)),
    personIdsByName:
      personIdsByName || mock(() => Promise.resolve([] as string[])),
    facesForPerson: facesForPerson || mock(() => Promise.resolve([] as Face[])),
    faceThumbnail: faceThumbnail || mock(() => Promise.resolve(null)),
    renamePerson: renamePerson || mock(() => Promise.resolve()),
    mergePeople: mergePeople || mock(() => Promise.resolve()),
    reassignFaces:
      reassignFaces || mock(() => Promise.resolve('mock-person')),
    hidePerson: hidePerson || mock(() => Promise.resolve()),
    setPersonThumbnail: setPersonThumbnail || mock(() => Promise.resolve()),
    allFaceAssetIds:
      allFaceAssetIds || mock(() => Promise.resolve([] as string[])),
    setFacesStatus: setFacesStatus || mock(() => Promise.resolve()),
    fetchFacesStatus:
      fetchFacesStatus ||
      mock(() => Promise.resolve(new Map<string, SyntheticStatus>())),
    assetIdsWithFacesStatus:
      assetIdsWithFacesStatus || mock(() => Promise.resolve([] as string[])),
    modelVersionsByAssets:
      modelVersionsByAssets ||
      mock(() => Promise.resolve(new Map<string, Set<string>>()))
  };
  return mockFaceStore;
}
