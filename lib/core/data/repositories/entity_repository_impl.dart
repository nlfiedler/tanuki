//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:meta/meta.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/data/sources/entity_remote_data_source.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/error/exceptions.dart';
import 'package:tanuki/core/error/failures.dart';

class EntityRepositoryImpl extends EntityRepository {
  final EntityRemoteDataSource remoteDataSource;

  EntityRepositoryImpl({
    @required this.remoteDataSource,
  });

  @override
  Future<Result<List<Location>, Failure>> getAllLocations() async {
    try {
      final locations = await remoteDataSource.getAllLocations();
      if (locations == null) {
        return Err(ServerFailure('got null result for locations'));
      }
      return Ok(locations);
    } on ServerException catch (e) {
      return Err(ServerFailure(e.toString()));
    }
  }

  @override
  Future<Result<List<Tag>, Failure>> getAllTags() async {
    try {
      final tags = await remoteDataSource.getAllTags();
      if (tags == null) {
        return Err(ServerFailure('got null result for tags'));
      }
      return Ok(tags);
    } on ServerException catch (e) {
      return Err(ServerFailure(e.toString()));
    }
  }

  @override
  Future<Result<List<Year>, Failure>> getAllYears() async {
    try {
      final years = await remoteDataSource.getAllYears();
      if (years == null) {
        return Err(ServerFailure('got null result for years'));
      }
      return Ok(years);
    } on ServerException catch (e) {
      return Err(ServerFailure(e.toString()));
    }
  }

  @override
  Future<Result<Asset, Failure>> getAsset(String id) async {
    try {
      final results = await remoteDataSource.getAsset(id);
      if (results == null) {
        return Err(ServerFailure('got null result for query'));
      }
      return Ok(results);
    } on ServerException catch (e) {
      return Err(ServerFailure(e.toString()));
    }
  }

  @override
  Future<Result<int, Failure>> getAssetCount() async {
    try {
      final count = await remoteDataSource.getAssetCount();
      if (count == null) {
        return Err(ServerFailure('got null result for count'));
      }
      return Ok(count);
    } on ServerException catch (e) {
      return Err(ServerFailure(e.toString()));
    }
  }

  @override
  Future<Result<QueryResults, Failure>> queryAssets(
    SearchParams params,
    int count,
    int offset,
  ) async {
    try {
      final results = await remoteDataSource.queryAssets(
        params,
        count,
        offset,
      );
      if (results == null) {
        return Err(ServerFailure('got null result for query'));
      }
      return Ok(results);
    } on ServerException catch (e) {
      return Err(ServerFailure(e.toString()));
    }
  }
}
