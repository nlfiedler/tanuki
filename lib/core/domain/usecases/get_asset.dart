//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:equatable/equatable.dart';
import 'package:meta/meta.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';
import 'package:tanuki/core/error/failures.dart';

class GetAsset implements UseCase<Asset, Params> {
  final EntityRepository repository;

  GetAsset(this.repository);

  @override
  Future<Result<Asset, Failure>> call(Params params) async {
    return await repository.getAsset(
      params.assetId,
    );
  }
}

class Params extends Equatable {
  final String assetId;

  Params({
    @required this.assetId,
  });

  @override
  List<Object> get props => [assetId];
}
