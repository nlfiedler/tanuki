//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:get_it/get_it.dart';
import 'entity_remote_data_source.dart';

void initDataSources(GetIt getIt) {
  getIt.registerLazySingleton<EntityRemoteDataSource>(
    () => EntityRemoteDataSourceImpl(client: getIt()),
  );
}
