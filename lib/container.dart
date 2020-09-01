//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:get_it/get_it.dart';
import 'package:graphql/client.dart';
import 'package:http/http.dart' as http;
import 'package:tanuki/core/data/repositories/container.dart';
import 'package:tanuki/core/data/sources/container.dart';
import 'package:tanuki/core/domain/usecases/container.dart';
import 'package:tanuki/environment_config.dart';
import 'package:tanuki/features/browse/preso/bloc/container.dart';
import 'package:tanuki/features/upload/preso/bloc/container.dart';

final getIt = GetIt.instance;

void init() {
  // bloc
  initBrowseBlocs(getIt);
  initUploadBlocs(getIt);

  // widgets

  initUseCases(getIt);
  initRepositories(getIt);
  initDataSources(getIt);

  // core

  // external
  getIt.registerLazySingleton(() {
    // seems a relative URL is not supported by the client package
    final uri = '${EnvironmentConfig.base_url}/graphql';
    return GraphQLClient(
      link: HttpLink(uri: uri),
      cache: InMemoryCache(),
    );
  });

  getIt.registerLazySingleton(() => http.Client());
}
