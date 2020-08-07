//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:get_it/get_it.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'entity_repository_impl.dart';

void initRepositories(GetIt getIt) {
  getIt.registerLazySingleton<EntityRepository>(
    () => EntityRepositoryImpl(remoteDataSource: getIt()),
  );
}
