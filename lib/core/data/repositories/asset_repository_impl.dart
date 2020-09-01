//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:typed_data';
import 'package:meta/meta.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/data/sources/asset_remote_data_source.dart';
import 'package:tanuki/core/domain/repositories/asset_repository.dart';
import 'package:tanuki/core/error/exceptions.dart';
import 'package:tanuki/core/error/failures.dart';

class AssetRepositoryImpl extends AssetRepository {
  final AssetRemoteDataSource remoteDataSource;

  AssetRepositoryImpl({
    @required this.remoteDataSource,
  });

  @override
  Future<Result<String, Failure>> uploadAsset(String filepath) async {
    try {
      final results = await remoteDataSource.uploadAsset(filepath);
      if (results == null) {
        return Err(ServerFailure('got null result for upload'));
      }
      return Ok(results);
    } on ServerException catch (e) {
      return Err(ServerFailure(e.toString()));
    }
  }

  @override
  Future<Result<String, Failure>> uploadAssetBytes(
    String filename,
    Uint8List contents,
  ) async {
    try {
      final results = await remoteDataSource.uploadAssetBytes(
        filename,
        contents,
      );
      if (results == null) {
        return Err(ServerFailure('got null result for upload'));
      }
      return Ok(results);
    } on ServerException catch (e) {
      return Err(ServerFailure(e.toString()));
    }
  }
}
