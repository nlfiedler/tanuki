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
import FindPending from 'tanuki/server/domain/usecases/FindPending.ts';
import GetLocationParts from 'tanuki/server/domain/usecases/GetLocationParts.ts';
import GetLocationRecords from 'tanuki/server/domain/usecases/GetLocationRecords.ts';
import GetTags from 'tanuki/server/domain/usecases/GetTags.ts';
import ImportAsset from 'tanuki/server/domain/usecases/ImportAsset.ts';
import SearchAssets from 'tanuki/server/domain/usecases/SearchAssets.ts';
import UpdateAsset from 'tanuki/server/domain/usecases/UpdateAsset.ts';
import { CouchDBRecordRepository } from 'tanuki/server/data/repositories/CouchDBRecordRepository.ts';
import { EnvSettingsRepository } from 'tanuki/server/data/repositories/EnvSettingsRepository.ts';
import { LocalBlobRepository } from 'tanuki/server/data/repositories/LocalBlobRepository.ts';

// create the injection container
const container = createContainer({
  injectionMode: InjectionMode.PROXY
});

container.register({
  // register the data repositories as classes
  settingsRepository: asClass(EnvSettingsRepository, { lifetime: Lifetime.SINGLETON }),
  recordRepository: asClass(CouchDBRecordRepository, { lifetime: Lifetime.SINGLETON }),
  blobRepository: asClass(LocalBlobRepository, { lifetime: Lifetime.SINGLETON }),

  // register the use cases as functions
  countAssets: asFunction(CountAssets),
  findPending: asFunction(FindPending),
  getLocationParts: asFunction(GetLocationParts),
  getLocationRecords: asFunction(GetLocationRecords),
  getTags: asFunction(GetTags),
  importAsset: asFunction(ImportAsset),
  searchAssets: asFunction(SearchAssets),
  updateAsset: asFunction(UpdateAsset),
});

export default container;
