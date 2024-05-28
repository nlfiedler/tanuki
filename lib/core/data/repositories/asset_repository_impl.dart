//
// Copyright (c) 2024 Nathan Fiedler
//
import 'dart:typed_data';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/data/sources/asset_remote_data_source.dart';
import 'package:tanuki/core/domain/repositories/asset_repository.dart';
import 'package:tanuki/core/error/exceptions.dart';
import 'package:tanuki/core/error/failures.dart';

class AssetRepositoryImpl extends AssetRepository {
  final AssetRemoteDataSource remoteDataSource;

  AssetRepositoryImpl({
    required this.remoteDataSource,
  });

  @override
  Future<Result<int, Failure>> ingestAssets() async {
    try {
      return Ok(await remoteDataSource.ingestAssets());
    } on ServerException catch (e) {
      return Err(ServerFailure(e.toString()));
    }
  }

  @override
  Future<Result<String, Failure>> replaceAssetBytes(
    String assetId,
    String filename,
    Uint8List contents,
  ) async {
    try {
      return Ok(await remoteDataSource.replaceAssetBytes(
        assetId,
        filename,
        contents,
      ));
    } on ServerException catch (e) {
      return Err(ServerFailure(e.toString()));
    }
  }

  @override
  Future<Result<String, Failure>> uploadAsset(String filepath) async {
    try {
      return Ok(await remoteDataSource.uploadAsset(filepath));
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
      return Ok(await remoteDataSource.uploadAssetBytes(
        filename,
        contents,
      ));
    } on ServerException catch (e) {
      return Err(ServerFailure(e.toString()));
    }
  }
}
