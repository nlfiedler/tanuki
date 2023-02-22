//
// Copyright (c) 2023 Nathan Fiedler
//
import 'dart:typed_data';
import 'package:equatable/equatable.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/repositories/asset_repository.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';
import 'package:tanuki/core/error/failures.dart';

class UploadAsset implements UseCase<String, Params> {
  final AssetRepository repository;

  UploadAsset(this.repository);

  @override
  Future<Result<String, Failure>> call(Params params) async {
    return await repository.uploadAssetBytes(params.filename, params.contents);
  }
}

class Params extends Equatable {
  final String filename;
  final Uint8List contents;

  const Params({
    required this.filename,
    required this.contents,
  });

  @override
  List<Object> get props => [filename];
}
