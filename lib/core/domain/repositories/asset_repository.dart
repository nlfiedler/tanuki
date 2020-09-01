//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:typed_data';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/error/failures.dart';

abstract class AssetRepository {
  /// Upload the given asset to the asset store.
  Future<Result<String, Failure>> uploadAsset(String filepath);

  /// Upload a file with the given name and contents to the asset store.
  Future<Result<String, Failure>> uploadAssetBytes(
    String filename,
    Uint8List contents,
  );
}
