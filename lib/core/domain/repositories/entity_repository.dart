//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/error/failures.dart';

abstract class EntityRepository {
  /// Retrieve the number of assets.
  Future<Result<int, Failure>> getAssetCount();
}
