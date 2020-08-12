//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:get_it/get_it.dart';
import 'get_all_locations.dart';
import 'get_all_tags.dart';
import 'get_all_years.dart';
import 'get_asset_count.dart';
import 'query_assets.dart';

void initUseCases(GetIt getIt) {
  getIt.registerLazySingleton(() => GetAllLocations(getIt()));
  getIt.registerLazySingleton(() => GetAllTags(getIt()));
  getIt.registerLazySingleton(() => GetAllYears(getIt()));
  getIt.registerLazySingleton(() => GetAssetCount(getIt()));
  getIt.registerLazySingleton(() => QueryAssets(getIt()));
}
