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
import ImportAsset from 'tanuki/server/domain/usecases/ImportAsset.ts';
import SearchAssets from 'tanuki/server/domain/usecases/SearchAssets.ts';
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
  importAsset: asFunction(ImportAsset),
  searchAssets: asFunction(SearchAssets),
});

export default container;
