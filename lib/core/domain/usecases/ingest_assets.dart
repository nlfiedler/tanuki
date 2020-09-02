//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/repositories/asset_repository.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';
import 'package:tanuki/core/error/failures.dart';

class IngestAssets implements UseCase<int, NoParams> {
  final AssetRepository repository;

  IngestAssets(this.repository);

  @override
  Future<Result<int, Failure>> call(NoParams params) async {
    return await repository.ingestAssets();
  }
}
