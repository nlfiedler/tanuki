//
// Copyright (c) 2024 Nathan Fiedler
//
import 'dart:typed_data';
import 'package:equatable/equatable.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/repositories/asset_repository.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';
import 'package:tanuki/core/error/failures.dart';

class ReplaceAsset implements UseCase<String, Params> {
  final AssetRepository repository;

  ReplaceAsset(this.repository);

  @override
  Future<Result<String, Failure>> call(Params params) async {
    return await repository.replaceAssetBytes(
        params.assetId, params.filename, params.contents);
  }
}

class Params extends Equatable {
  final String assetId;
  final String filename;
  final Uint8List contents;

  const Params({
    required this.assetId,
    required this.filename,
    required this.contents,
  });

  @override
  List<Object> get props => [assetId];
}
