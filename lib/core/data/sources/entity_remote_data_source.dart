//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:graphql/client.dart' as gql;
import 'package:meta/meta.dart';
import 'package:tanuki/core/data/models/attributes_model.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/error/exceptions.dart';

abstract class EntityRemoteDataSource {
  Future<List<Location>> getAllLocations();
  Future<List<Year>> getAllYears();
  Future<int> getAssetCount();
}

class EntityRemoteDataSourceImpl extends EntityRemoteDataSource {
  final gql.GraphQLClient client;

  EntityRemoteDataSourceImpl({@required this.client});

  @override
  Future<List<Location>> getAllLocations() async {
    final query = r'''
      query {
        locations {
          label
          count
        }
      }
    ''';
    final queryOptions = gql.QueryOptions(
      documentNode: gql.gql(query),
      fetchPolicy: gql.FetchPolicy.noCache,
    );
    final gql.QueryResult result = await client.query(queryOptions);
    if (result.hasException) {
      throw ServerException(result.exception.toString());
    }
    final List<dynamic> locations = result.data['locations'] as List<dynamic>;
    if (locations == null) {
      return null;
    }
    final List<LocationModel> results = List.from(
      locations.map<LocationModel>((e) {
        return LocationModel.fromJson(e);
      }),
    );
    return results;
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
    final queryOptions = gql.QueryOptions(
      documentNode: gql.gql(query),
      fetchPolicy: gql.FetchPolicy.noCache,
    );
    final gql.QueryResult result = await client.query(queryOptions);
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

  @override
  Future<int> getAssetCount() async {
    final query = r'''
      query {
        count
      }
    ''';
    final queryOptions = gql.QueryOptions(
      documentNode: gql.gql(query),
      fetchPolicy: gql.FetchPolicy.noCache,
    );
    final gql.QueryResult result = await client.query(queryOptions);
    if (result.hasException) {
      throw ServerException(result.exception.toString());
    }
    final count = result.data['count'];
    return count == null ? null : count as int;
  }
}
