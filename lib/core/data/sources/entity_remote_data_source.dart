//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:graphql/client.dart';
import 'package:meta/meta.dart';
import 'package:tanuki/core/error/exceptions.dart';

abstract class EntityRemoteDataSource {
  Future<int> getAssetCount();
}

class EntityRemoteDataSourceImpl extends EntityRemoteDataSource {
  final GraphQLClient client;

  EntityRemoteDataSourceImpl({@required this.client});

  @override
  Future<int> getAssetCount() async {
    final query = r'''
      query {
        count
      }
    ''';
    final queryOptions = QueryOptions(
      documentNode: gql(query),
      fetchPolicy: FetchPolicy.noCache,
    );
    final QueryResult result = await client.query(queryOptions);
    if (result.hasException) {
      throw ServerException(result.exception.toString());
    }
    final count = result.data['count'];
    return count == null ? null : count as int;
  }
}
