//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/core/error/failures.dart';

abstract class EntityRepository {
  /// Retrieve all of the locations and their counts.
  Future<Result<List<Location>, Failure>> getAllLocations();

  /// Retrieve all of the tags and their counts.
  Future<Result<List<Tag>, Failure>> getAllTags();

  /// Retrieve all of the years and their counts.
  Future<Result<List<Year>, Failure>> getAllYears();

  /// Retrieve the number of assets.
  Future<Result<int, Failure>> getAssetCount();

  /// Query for the assets matching the given parameters.
  Future<Result<QueryResults, Failure>> queryAssets(
    SearchParams params,
    int count,
    int offset,
  );
}
