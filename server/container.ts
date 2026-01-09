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
import { CouchDBRecordRepository } from 'tanuki/server/data/repositories/couchdb-record-repository.ts';
import { SqliteRecordRepository } from 'tanuki/server/data/repositories/sqlite-record-repository.ts';
import { DummyLocationRepository } from 'tanuki/server/data/repositories/dummy-location-repository.ts';
import { EnvSettingsRepository } from 'tanuki/server/data/repositories/env-settings-repository.ts';
import { GoogleLocationRepository } from 'tanuki/server/data/repositories/google-location-repository.ts';
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
}

container.register({
  // register the data repositories as classes
  settingsRepository: asClass(EnvSettingsRepository, {
    lifetime: Lifetime.SINGLETON
  }),
  recordRepository: asClass(RecordRepository, {
    lifetime: Lifetime.SINGLETON
  }),
  blobRepository: asClass(BlobRepository, {
    lifetime: Lifetime.SINGLETON
  }),
  locationRepository: asClass(LocationRepository, {
    lifetime: Lifetime.SINGLETON
  }),

  // register the use cases as functions
  countAssets: asFunction(CountAssets),
  dumpAssets: asFunction(DumpAssets),
  editAssets: asFunction(EditAssets),
  findPending: asFunction(FindPending),
  getAsset: asFunction(GetAsset),
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
  updateAsset: asFunction(UpdateAsset)
});

export default container;
