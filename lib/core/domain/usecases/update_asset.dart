//
// Copyright (c) 2023 Nathan Fiedler
//
import 'package:equatable/equatable.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/domain/entities/input.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';
import 'package:tanuki/core/error/failures.dart';

class UpdateAsset implements UseCase<Asset, Params> {
  final EntityRepository repository;

  UpdateAsset(this.repository);

  @override
  Future<Result<Asset, Failure>> call(Params params) async {
    return await repository.updateAsset(params.asset);
  }
}

class Params extends Equatable {
  final AssetInputId asset;

  const Params({
    required this.asset,
  });

  @override
  List<Object> get props => [asset];
}
