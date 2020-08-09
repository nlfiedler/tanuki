//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:meta/meta.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/data/sources/entity_remote_data_source.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
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
  Future<Result<int, Failure>> getAssetCount() async {
    try {
      final configuration = await remoteDataSource.getAssetCount();
      if (configuration == null) {
        return Err(ServerFailure('got null result for configuration'));
      }
      return Ok(configuration);
    } on ServerException catch (e) {
      return Err(ServerFailure(e.toString()));
    }
  }
}
