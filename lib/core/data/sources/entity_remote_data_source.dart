//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:graphql/client.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/data/models/asset_model.dart';
import 'package:tanuki/core/data/models/attributes_model.dart';
import 'package:tanuki/core/data/models/input_model.dart';
import 'package:tanuki/core/data/models/search_model.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/entities/input.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/core/error/exceptions.dart' as err;

abstract class EntityRemoteDataSource {
  Future<int> bulkUpdate(List<AssetInputId> assets);
  Future<List<Location>> getAllLocations();
  Future<List<Tag>> getAllTags();
  Future<List<Year>> getAllYears();
  Future<List<AssetLocation>> getAssetLocations();
  Future<Asset?> getAsset(String id);
  Future<int> getAssetCount();
  Future<QueryResults?> queryAssets(
    SearchParams params,
    int count,
    int offset,
  );
  Future<QueryResults?> queryRecents(
    Option<DateTime> since,
    Option<int> count,
    Option<int> offset,
  );
  Future<Asset?> updateAsset(AssetInputId asset);
}

class EntityRemoteDataSourceImpl extends EntityRemoteDataSource {
  final GraphQLClient client;

  EntityRemoteDataSourceImpl({required this.client});

  @override
  Future<int> bulkUpdate(List<AssetInputId> assets) async {
    const query = r'''
      mutation BulkUpdate($assets: [AssetInputId!]!) {
        bulkUpdate(assets: $assets)
      }
    ''';
    final models = List.of(
      assets.map((e) => AssetInputIdModel.from(e).toJson()),
    );
    final mutationOptions = MutationOptions(
      document: gql(query),
      variables: <String, dynamic>{
        'assets': models,
      },
    );
    final QueryResult result = await client.mutate(mutationOptions);
    if (result.hasException) {
      throw err.ServerException(result.exception.toString());
    }
    return (result.data?['bulkUpdate'] ?? 0) as int;
  }

  @override
  Future<List<Location>> getAllLocations() async {
    const query = r'''
      query {
        locations() {
          label
          count
        }
      }
    ''';
    final queryOptions = QueryOptions(
      document: gql(query),
      fetchPolicy: FetchPolicy.noCache,
    );
    final QueryResult result = await client.query(queryOptions);
    if (result.hasException) {
      throw err.ServerException(result.exception.toString());
    }
    if (result.data?['locations'] == null) {
      return [];
    }
    final List<dynamic> locations = result.data?['locations'] as List<dynamic>;
    final List<LocationModel> results = List.from(
      locations.map<LocationModel>((e) {
        return LocationModel.fromJson(e);
      }),
    );
    return results;
  }

  @override
  Future<List<Tag>> getAllTags() async {
    const query = r'''
      query {
        tags {
          label
          count
        }
      }
    ''';
    final queryOptions = QueryOptions(
      document: gql(query),
      fetchPolicy: FetchPolicy.noCache,
    );
    final QueryResult result = await client.query(queryOptions);
    if (result.hasException) {
      throw err.ServerException(result.exception.toString());
    }
    if (result.data?['tags'] == null) {
      return [];
    }
    final List<dynamic> tags = result.data?['tags'] as List<dynamic>;
    final List<TagModel> results = List.from(
      tags.map<TagModel>((e) {
        return TagModel.fromJson(e);
      }),
    );
    return results;
  }

  @override
  Future<List<Year>> getAllYears() async {
    const query = r'''
      query {
        years {
          label
          count
        }
      }
    ''';
    final queryOptions = QueryOptions(
      document: gql(query),
      fetchPolicy: FetchPolicy.noCache,
    );
    final QueryResult result = await client.query(queryOptions);
    if (result.hasException) {
      throw err.ServerException(result.exception.toString());
    }
    if (result.data?['years'] == null) {
      return [];
    }
    final List<dynamic> years = result.data?['years'] as List<dynamic>;
    final List<YearModel> results = List.from(
      years.map<YearModel>((e) {
        return YearModel.fromJson(e);
      }),
    );
    return results;
  }

  @override
  Future<List<AssetLocation>> getAssetLocations() async {
    const query = r'''
      query {
        allLocations() {
          label
          city
          region
        }
      }
    ''';
    final queryOptions = QueryOptions(
      document: gql(query),
      fetchPolicy: FetchPolicy.noCache,
    );
    final QueryResult result = await client.query(queryOptions);
    if (result.hasException) {
      throw err.ServerException(result.exception.toString());
    }
    if (result.data?['allLocations'] == null) {
      return [];
    }
    final List<dynamic> locations =
        result.data?['allLocations'] as List<dynamic>;
    final List<AssetLocationModel> results = List.from(
      locations.map<AssetLocationModel>((e) {
        return AssetLocationModel.fromJson(e);
      }),
    );
    return results;
  }

  @override
  Future<Asset?> getAsset(String id) async {
    const query = r'''
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
          location { label city region }
        }
      }
    ''';
    final queryOptions = QueryOptions(
      document: gql(query),
      variables: <String, dynamic>{
        'identifier': id,
      },
      fetchPolicy: FetchPolicy.noCache,
    );
    final QueryResult result = await client.query(queryOptions);
    if (result.hasException) {
      throw err.ServerException(result.exception.toString());
    }
    if (result.data?['asset'] == null) {
      return null;
    }
    return AssetModel.fromJson(result.data?['asset'] as Map<String, dynamic>);
  }

  @override
  Future<int> getAssetCount() async {
    const query = r'''
      query {
        count
      }
    ''';
    final queryOptions = QueryOptions(
      document: gql(query),
      fetchPolicy: FetchPolicy.noCache,
    );
    final QueryResult result = await client.query(queryOptions);
    if (result.hasException) {
      throw err.ServerException(result.exception.toString());
    }
    return (result.data?['count'] ?? 0) as int;
  }

  @override
  Future<QueryResults?> queryAssets(
    SearchParams params,
    int count,
    int offset,
  ) async {
    const query = r'''
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
    final queryOptions = QueryOptions(
      document: gql(query),
      variables: <String, dynamic>{
        'params': encodedParams,
        'count': count,
        'offset': offset,
      },
      fetchPolicy: FetchPolicy.noCache,
    );
    final QueryResult result = await client.query(queryOptions);
    if (result.hasException) {
      throw err.ServerException(result.exception.toString());
    }
    if (result.data?['search'] == null) {
      return null;
    }
    return QueryResultsModel.fromJson(
      result.data?['search'] as Map<String, dynamic>,
    );
  }

  @override
  Future<QueryResults?> queryRecents(
    Option<DateTime> since,
    Option<int> count,
    Option<int> offset,
  ) async {
    final validDate = since.mapOr((v) => v.isUtc, true);
    if (!validDate) {
      throw const err.ServerException('since must be a UTC date/time');
    }
    const query = r'''
      query Recent($since: DateTimeUtc, $count: Int, $offset: Int) {
        recent(since: $since, count: $count, offset: $offset) {
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
    final queryOptions = QueryOptions(
      document: gql(query),
      variables: <String, dynamic>{
        'since': since.mapOr((v) => v.toIso8601String(), null),
        'count': count.toNullable(),
        'offset': offset.toNullable(),
      },
      fetchPolicy: FetchPolicy.noCache,
    );
    final QueryResult result = await client.query(queryOptions);
    if (result.hasException) {
      throw err.ServerException(result.exception.toString());
    }
    if (result.data?['recent'] == null) {
      return null;
    }
    return QueryResultsModel.fromJson(
      result.data?['recent'] as Map<String, dynamic>,
    );
  }

  @override
  Future<Asset?> updateAsset(AssetInputId asset) async {
    const query = r'''
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
          location { label city region }
        }
      }
    ''';
    final model = AssetInputModel.from(asset.input).toJson();
    final mutationOptions = MutationOptions(
      document: gql(query),
      variables: <String, dynamic>{
        'identifier': asset.id,
        'input': model,
      },
    );
    final QueryResult result = await client.mutate(mutationOptions);
    if (result.hasException) {
      throw err.ServerException(result.exception.toString());
    }
    if (result.data?['update'] == null) {
      return null;
    }
    return AssetModel.fromJson(result.data?['update'] as Map<String, dynamic>);
  }
}
