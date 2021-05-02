//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:graphql/client.dart';
import 'package:http/http.dart' as http;
import 'package:tanuki/core/data/repositories/asset_repository_impl.dart';
import 'package:tanuki/core/data/repositories/entity_repository_impl.dart';
import 'package:tanuki/core/data/sources/asset_remote_data_source.dart';
import 'package:tanuki/core/data/sources/entity_remote_data_source.dart';
import 'package:tanuki/core/domain/repositories/asset_repository.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/environment_config.dart';

final httpClientProvider = Provider<http.Client>((_) => http.Client());

final graphqlProvider = Provider<GraphQLClient>((ref) {
  final uri = '${EnvironmentConfig.base_url}/graphql';
  return GraphQLClient(
    link: HttpLink(uri),
    cache: GraphQLCache(),
  );
});

final assetDataSourceProvider = Provider<AssetRemoteDataSource>((ref) {
  return AssetRemoteDataSourceImpl(
    httpClient: ref.read(httpClientProvider),
    baseUrl: EnvironmentConfig.base_url,
    gqlClient: ref.read(graphqlProvider),
  );
});

final entityDataSourceProvider = Provider<EntityRemoteDataSource>((ref) {
  return EntityRemoteDataSourceImpl(
    client: ref.read(graphqlProvider),
  );
});

final assetRepositoryProvider = Provider<AssetRepository>(
  (ref) => AssetRepositoryImpl(
    remoteDataSource: ref.read(assetDataSourceProvider),
  ),
);

final entityRepositoryProvider = Provider<EntityRepository>(
  (ref) => EntityRepositoryImpl(
    remoteDataSource: ref.read(entityDataSourceProvider),
  ),
);
