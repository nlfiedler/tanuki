//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/entities/input.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/core/error/failures.dart';

abstract class EntityRepository {
  /// Update multiple asset records in the repository.
  Future<Result<int, Failure>> bulkUpdate(List<AssetInputId> assets);

  /// Retrieve all of the locations and their counts.
  Future<Result<List<Location>, Failure>> getAllLocations(bool raw);

  /// Retrieve all of the tags and their counts.
  Future<Result<List<Tag>, Failure>> getAllTags();

  /// Retrieve all of the years and their counts.
  Future<Result<List<Year>, Failure>> getAllYears();

  /// Retrieve the asset with the given unique identifier.
  Future<Result<Asset, Failure>> getAsset(String id);

  /// Retrieve the number of assets.
  Future<Result<int, Failure>> getAssetCount();

  /// Query for the assets matching the given parameters.
  Future<Result<QueryResults, Failure>> queryAssets(
    SearchParams params,
    int count,
    int offset,
  );

  /// Query for the recent imports since the given date/time.
  Future<Result<QueryResults, Failure>> queryRecents(
    Option<DateTime> since,
    Option<int> count,
    Option<int> offset,
  );

  /// Update a single asset record in the repository.
  ///
  /// Returns the updated asset record, which may differ from the input as the
  /// backend may merge, sort, or otherwise modify the entity data.
  Future<Result<Asset, Failure>> updateAsset(AssetInputId asset);
}
