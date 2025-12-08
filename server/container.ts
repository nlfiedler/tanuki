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
import CountAssets from 'tanuki/server/domain/usecases/CountAssets.ts';
import DumpAssets from 'tanuki/server/domain/usecases/DumpAssets.ts';
import FindPending from 'tanuki/server/domain/usecases/FindPending.ts';
import GetAsset from 'tanuki/server/domain/usecases/GetAsset.ts';
import GetLocationParts from 'tanuki/server/domain/usecases/GetLocationParts.ts';
import GetLocationRecords from 'tanuki/server/domain/usecases/GetLocationRecords.ts';
import GetMediaTypes from 'tanuki/server/domain/usecases/GetMediaTypes.ts';
import GetTags from 'tanuki/server/domain/usecases/GetTags.ts';
import GetYears from 'tanuki/server/domain/usecases/GetYears.ts';
import LoadAssets from 'tanuki/server/domain/usecases/LoadAssets.ts';
import ImportAsset from 'tanuki/server/domain/usecases/ImportAsset.ts';
import ImportUploads from 'tanuki/server/domain/usecases/ImportUploads.ts';
import SearchAssets from 'tanuki/server/domain/usecases/SearchAssets.ts';
import UpdateAsset from 'tanuki/server/domain/usecases/UpdateAsset.ts';
import { CouchDBRecordRepository } from 'tanuki/server/data/repositories/CouchDBRecordRepository.ts';
import { DummyLocationRepository } from 'tanuki/server/data/repositories/DummyLocationRepository.ts';
import { EnvSettingsRepository } from 'tanuki/server/data/repositories/EnvSettingsRepository.ts';
import { GoogleLocationRepository } from 'tanuki/server/data/repositories/GoogleLocationRepository.ts';
import { LocalBlobRepository } from 'tanuki/server/data/repositories/LocalBlobRepository.ts';

// create the injection container
const container = createContainer({
  injectionMode: InjectionMode.PROXY
});

let LocationRepository: any = DummyLocationRepository;
if ('GOOGLE_MAPS_API_KEY' in process.env) {
  LocationRepository = GoogleLocationRepository;
}

container.register({
  // register the data repositories as classes
  settingsRepository: asClass(EnvSettingsRepository, { lifetime: Lifetime.SINGLETON }),
  recordRepository: asClass(CouchDBRecordRepository, { lifetime: Lifetime.SINGLETON }),
  blobRepository: asClass(LocalBlobRepository, { lifetime: Lifetime.SINGLETON }),
  locationRepository: asClass(LocationRepository, { lifetime: Lifetime.SINGLETON }),

  // register the use cases as functions
  countAssets: asFunction(CountAssets),
  dumpAssets: asFunction(DumpAssets),
  findPending: asFunction(FindPending),
  getAsset: asFunction(GetAsset),
  getLocationParts: asFunction(GetLocationParts),
  getLocationRecords: asFunction(GetLocationRecords),
  getMediaTypes: asFunction(GetMediaTypes),
  getTags: asFunction(GetTags),
  getYears: asFunction(GetYears),
  importAsset: asFunction(ImportAsset),
  importUploads: asFunction(ImportUploads),
  loadAssets: asFunction(LoadAssets),
  searchAssets: asFunction(SearchAssets),
  updateAsset: asFunction(UpdateAsset),
});

export default container;
