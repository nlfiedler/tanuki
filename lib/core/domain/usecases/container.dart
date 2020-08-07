//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:get_it/get_it.dart';
import 'get_asset_count.dart';

void initUseCases(GetIt getIt) {
  getIt.registerLazySingleton(() => GetAssetCount(getIt()));
}
