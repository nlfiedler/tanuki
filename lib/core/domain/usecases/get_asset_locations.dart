//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';
import 'package:tanuki/core/error/failures.dart';

class GetAssetLocations implements UseCase<List<AssetLocation>, NoParams> {
  final EntityRepository repository;

  GetAssetLocations(this.repository);

  @override
  Future<Result<List<AssetLocation>, Failure>> call(NoParams params) async {
    return await repository.getAssetLocations();
  }
}
