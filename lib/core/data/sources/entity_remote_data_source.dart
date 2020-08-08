//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:graphql/client.dart';
import 'package:meta/meta.dart';
import 'package:tanuki/core/data/models/attributes_model.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/error/exceptions.dart';

abstract class EntityRemoteDataSource {
  Future<int> getAssetCount();
  Future<List<Year>> getAllYears();
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

  @override
  Future<List<Year>> getAllYears() async {
    final query = r'''
      query {
        years {
          label
          count
        }
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
    final List<dynamic> years = result.data['years'] as List<dynamic>;
    if (years == null) {
      return null;
    }
    final List<YearModel> results = List.from(
      years.map<YearModel>((e) {
        return YearModel.fromJson(e);
      }),
    );
    return results;
  }
}
