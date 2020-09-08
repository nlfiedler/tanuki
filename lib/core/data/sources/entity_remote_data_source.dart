//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:graphql/client.dart' as gql;
import 'package:meta/meta.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/data/models/asset_model.dart';
import 'package:tanuki/core/data/models/attributes_model.dart';
import 'package:tanuki/core/data/models/input_model.dart';
import 'package:tanuki/core/data/models/search_model.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/entities/input.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/core/error/exceptions.dart';

abstract class EntityRemoteDataSource {
  Future<int> bulkUpdate(List<AssetInputId> assets);
  Future<List<Location>> getAllLocations();
  Future<List<Tag>> getAllTags();
  Future<List<Year>> getAllYears();
  Future<Asset> getAsset(String id);
  Future<int> getAssetCount();
  Future<QueryResults> queryAssets(
    SearchParams params,
    int count,
    int offset,
  );
  Future<QueryResults> queryRecents(Option<DateTime> since);
  Future<Asset> updateAsset(AssetInputId asset);
}

class EntityRemoteDataSourceImpl extends EntityRemoteDataSource {
  final gql.GraphQLClient client;

  EntityRemoteDataSourceImpl({@required this.client});

  @override
  Future<int> bulkUpdate(List<AssetInputId> assets) async {
    final query = r'''
      mutation BulkUpdate($assets: [AssetInputId!]!) {
        bulkUpdate(assets: $assets)
      }
    ''';
    final models = List.of(
      assets.map((e) => AssetInputIdModel.from(e).toJson()),
    );
    final mutationOptions = gql.MutationOptions(
      documentNode: gql.gql(query),
      variables: <String, dynamic>{
        'assets': models,
      },
    );
    final gql.QueryResult result = await client.mutate(mutationOptions);
    if (result.hasException) {
      throw ServerException(result.exception.toString());
    }
    final count = result.data['bulkUpdate'];
    return count == null ? null : count as int;
  }

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
  Future<List<Tag>> getAllTags() async {
    final query = r'''
      query {
        tags {
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
    final List<dynamic> tags = result.data['tags'] as List<dynamic>;
    if (tags == null) {
      return null;
    }
    final List<TagModel> results = List.from(
      tags.map<TagModel>((e) {
        return TagModel.fromJson(e);
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
  Future<Asset> getAsset(String id) async {
    final query = r'''
      query Fetch($identifier: String!) {
        asset(id: $identifier) {
          id
          checksum
          filename
          filesize
          datetime
          mimetype
          tags
          userdate
          caption
          location
        }
      }
    ''';
    final queryOptions = gql.QueryOptions(
      documentNode: gql.gql(query),
      variables: <String, dynamic>{
        'identifier': id,
      },
      fetchPolicy: gql.FetchPolicy.noCache,
    );
    final gql.QueryResult result = await client.query(queryOptions);
    if (result.hasException) {
      throw ServerException(result.exception.toString());
    }
    final Map<String, dynamic> object =
        result.data['asset'] as Map<String, dynamic>;
    return object == null ? null : AssetModel.fromJson(object);
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

  @override
  Future<QueryResults> queryAssets(
    SearchParams params,
    int count,
    int offset,
  ) async {
    final query = r'''
      query Search($params: SearchParams!, $count: Int, $offset: Int) {
        search(params: $params, count: $count, offset: $offset) {
          results {
            id
            datetime
            filename
            location
            mimetype
          }
          count
        }
      }
    ''';
    final paramsModel = SearchParamsModel.from(params);
    final encodedParams = paramsModel.toJson();
    final queryOptions = gql.QueryOptions(
      documentNode: gql.gql(query),
      variables: <String, dynamic>{
        'params': encodedParams,
        'count': count,
        'offset': offset,
      },
      fetchPolicy: gql.FetchPolicy.noCache,
    );
    final gql.QueryResult result = await client.query(queryOptions);
    if (result.hasException) {
      throw ServerException(result.exception.toString());
    }
    final Map<String, dynamic> object =
        result.data['search'] as Map<String, dynamic>;
    return object == null ? null : QueryResultsModel.fromJson(object);
  }

  @override
  Future<QueryResults> queryRecents(Option<DateTime> since) async {
    final validDate = since.mapOr((v) => v.isUtc, true);
    if (!validDate) {
      throw ServerException('since must be a UTC date/time');
    }
    final query = r'''
      query Recent($since: DateTimeUtc) {
        recent(since: $since) {
          results {
            id
            datetime
            filename
            location
            mimetype
          }
          count
        }
      }
    ''';
    final queryOptions = gql.QueryOptions(
      documentNode: gql.gql(query),
      variables: <String, dynamic>{
        'since': since.mapOr((v) => v.toIso8601String(), null),
      },
      fetchPolicy: gql.FetchPolicy.noCache,
    );
    final gql.QueryResult result = await client.query(queryOptions);
    if (result.hasException) {
      throw ServerException(result.exception.toString());
    }
    final Map<String, dynamic> object =
        result.data['recent'] as Map<String, dynamic>;
    return object == null ? null : QueryResultsModel.fromJson(object);
  }

  @override
  Future<Asset> updateAsset(AssetInputId asset) async {
    final query = r'''
      mutation Update($identifier: String!, $input: AssetInput!) {
        update(id: $identifier, asset: $input) {
          id
          checksum
          filename
          filesize
          datetime
          mimetype
          tags
          userdate
          caption
          location
        }
      }
    ''';
    final model = AssetInputModel.from(asset.input).toJson();
    final mutationOptions = gql.MutationOptions(
      documentNode: gql.gql(query),
      variables: <String, dynamic>{
        'identifier': asset.id,
        'assets': model,
      },
    );
    final gql.QueryResult result = await client.mutate(mutationOptions);
    if (result.hasException) {
      throw ServerException(result.exception.toString());
    }
    final Map<String, dynamic> object =
        result.data['update'] as Map<String, dynamic>;
    return object == null ? null : AssetModel.fromJson(object);
  }
}
