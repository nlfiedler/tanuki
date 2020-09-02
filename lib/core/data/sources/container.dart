//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:get_it/get_it.dart';
import 'package:tanuki/environment_config.dart';
import 'asset_remote_data_source.dart';
import 'entity_remote_data_source.dart';

void initDataSources(GetIt getIt) {
  getIt.registerLazySingleton<AssetRemoteDataSource>(
    () => AssetRemoteDataSourceImpl(
      httpClient: getIt(),
      baseUrl: EnvironmentConfig.base_url,
      gqlClient: getIt(),
    ),
  );
  getIt.registerLazySingleton<EntityRemoteDataSource>(
    () => EntityRemoteDataSourceImpl(client: getIt()),
  );
}
