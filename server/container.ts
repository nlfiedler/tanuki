//
// Copyright (c) 2025 Nathan Fiedler
//
import {
  asClass,
  asFunction,
  createContainer,
  InjectionMode,
  Lifetime
} from 'awilix';
import CountAssets from 'tanuki/server/domain/usecases/count-assets.ts';
import DumpAssets from 'tanuki/server/domain/usecases/dump-assets.ts';
import EditAssets from 'tanuki/server/domain/usecases/edit-assets.ts';
import FindPending from 'tanuki/server/domain/usecases/find-pending.ts';
import GetAsset from 'tanuki/server/domain/usecases/get-asset.ts';
import GetAssetByDigest from 'tanuki/server/domain/usecases/get-asset-by-digest.ts';
import GetAssetTags from 'tanuki/server/domain/usecases/get-asset-tags.ts';
import GetLocationParts from 'tanuki/server/domain/usecases/get-location-parts.ts';
import GetLocationRecords from 'tanuki/server/domain/usecases/get-location-records.ts';
import GetLocationValues from 'tanuki/server/domain/usecases/get-location-values.ts';
import GetMediaTypes from 'tanuki/server/domain/usecases/get-media-types.ts';
import GetTags from 'tanuki/server/domain/usecases/get-tags.ts';
import GetYears from 'tanuki/server/domain/usecases/get-years.ts';
import LoadAssets from 'tanuki/server/domain/usecases/load-assets.ts';
import ImportAsset from 'tanuki/server/domain/usecases/import-asset.ts';
import ImportUploads from 'tanuki/server/domain/usecases/import-uploads.ts';
import ReplaceAsset from 'tanuki/server/domain/usecases/replace-asset.ts';
import ScanAssets from 'tanuki/server/domain/usecases/scan-assets.ts';
import SearchAssets from 'tanuki/server/domain/usecases/search-assets.ts';
import UpdateAsset from 'tanuki/server/domain/usecases/update-asset.ts';
import UpdateVideoDates from 'tanuki/server/domain/usecases/update-video-dates.ts';
import BackfillImageMetadata from 'tanuki/server/domain/usecases/backfill-image-metadata.ts';
import BackfillVideoMetadata from 'tanuki/server/domain/usecases/backfill-video-metadata.ts';
import BackfillLabels from 'tanuki/server/domain/usecases/backfill-labels.ts';
import RetrySyntheticJobs from 'tanuki/server/domain/usecases/retry-synthetic-jobs.ts';
import GetLabels from 'tanuki/server/domain/usecases/get-labels.ts';
import AssetsByLabel from 'tanuki/server/domain/usecases/assets-by-label.ts';
import FixOriginalDates from 'tanuki/server/domain/usecases/fix-original-dates.ts';
import { CouchDBRecordRepository } from 'tanuki/server/data/repositories/couchdb-record-repository.ts';
import { PouchDBRecordRepository } from 'tanuki/server/data/repositories/pouchdb-record-repository.ts';
import { SqliteRecordRepository } from 'tanuki/server/data/repositories/sqlite-record-repository.ts';
import { SqliteFaceStore } from 'tanuki/server/data/repositories/sqlite-face-store.ts';
import { LocalSyntheticDetector } from 'tanuki/server/data/repositories/local-synthetic-detector.ts';
import { DetectingSyntheticJobProcessor } from 'tanuki/server/data/repositories/detecting-synthetic-job-processor.ts';
import { SyntheticWorkerPool } from 'tanuki/server/domain/services/synthetic-worker-pool.ts';
import { DummyLocationRepository } from 'tanuki/server/data/repositories/dummy-location-repository.ts';
import { EnvSettingsRepository } from 'tanuki/server/data/repositories/env-settings-repository.ts';
import { GoogleLocationRepository } from 'tanuki/server/data/repositories/google-location-repository.ts';
import { MemorySearchRepository } from 'tanuki/server/data/repositories/memory-search-repository.ts';
import { LocalBlobRepository } from './data/repositories/local-blob-repository';
import { NamazuBlobRepository } from './data/repositories/namazu-blob-repository';

// create the injection container
const container = createContainer({
  injectionMode: InjectionMode.PROXY
});

// assume local blob repository unless specified otherwise
let BlobRepository: any = LocalBlobRepository;
if ('NAMAZU_URL' in process.env) {
  BlobRepository = NamazuBlobRepository;
}

let LocationRepository: any = DummyLocationRepository;
if ('GOOGLE_MAPS_API_KEY' in process.env) {
  LocationRepository = GoogleLocationRepository;
}

// assume CouchDB unless specified otherwise
let RecordRepository: any = CouchDBRecordRepository;
if ('SQLITE_DBPATH' in process.env) {
  RecordRepository = SqliteRecordRepository;
} else if ('POUCHDB_PATH' in process.env) {
  RecordRepository = PouchDBRecordRepository;
}

container.register({
  // register the data repositories as classes
  settingsRepository: asClass(EnvSettingsRepository, {
    lifetime: Lifetime.SINGLETON
  }),
  recordRepository: asClass(RecordRepository, {
    lifetime: Lifetime.SINGLETON
  }),
  // The face store is always SQLite, regardless of the asset record backend:
  // embeddings, face crops, and the job queue need relational/BLOB semantics.
  faceStore: asClass(SqliteFaceStore, {
    lifetime: Lifetime.SINGLETON
  }),
  // Label detector (MobileNetV2 via onnxruntime-node), reading bytes from
  // whichever blob store is configured. The Namazu HTTP push-down is a future
  // optimization that would swap this registration.
  syntheticDetector: asClass(LocalSyntheticDetector, {
    lifetime: Lifetime.SINGLETON
  }),
  // Drains the synthetic_jobs queue, running the detector and persisting labels.
  syntheticJobProcessor: asClass(DetectingSyntheticJobProcessor, {
    lifetime: Lifetime.SINGLETON
  }),
  syntheticWorkerPool: asClass(SyntheticWorkerPool, {
    lifetime: Lifetime.SINGLETON
  }),
  blobRepository: asClass(BlobRepository, {
    lifetime: Lifetime.SINGLETON
  }),
  locationRepository: asClass(LocationRepository, {
    lifetime: Lifetime.SINGLETON
  }),
  searchRepository: asClass(MemorySearchRepository, {
    lifetime: Lifetime.SINGLETON
  }),

  // register the use cases as functions
  countAssets: asFunction(CountAssets),
  dumpAssets: asFunction(DumpAssets),
  editAssets: asFunction(EditAssets),
  findPending: asFunction(FindPending),
  getAsset: asFunction(GetAsset),
  getAssetByDigest: asFunction(GetAssetByDigest),
  getAssetTags: asFunction(GetAssetTags),
  getLocationParts: asFunction(GetLocationParts),
  getLocationRecords: asFunction(GetLocationRecords),
  getLocationValues: asFunction(GetLocationValues),
  getMediaTypes: asFunction(GetMediaTypes),
  getTags: asFunction(GetTags),
  getYears: asFunction(GetYears),
  importAsset: asFunction(ImportAsset),
  importUploads: asFunction(ImportUploads),
  loadAssets: asFunction(LoadAssets),
  replaceAsset: asFunction(ReplaceAsset),
  scanAssets: asFunction(ScanAssets),
  searchAssets: asFunction(SearchAssets),
  updateAsset: asFunction(UpdateAsset),
  updateVideoDates: asFunction(UpdateVideoDates),
  backfillImageMetadata: asFunction(BackfillImageMetadata),
  backfillVideoMetadata: asFunction(BackfillVideoMetadata),
  backfillLabels: asFunction(BackfillLabels),
  retrySyntheticJobs: asFunction(RetrySyntheticJobs),
  getLabels: asFunction(GetLabels),
  assetsByLabel: asFunction(AssetsByLabel),
  fixOriginalDates: asFunction(FixOriginalDates)
});

export default container;
