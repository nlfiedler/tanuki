//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';
import 'package:tanuki/core/error/failures.dart';

class GetAssetCount implements UseCase<int, NoParams> {
  final EntityRepository repository;

  GetAssetCount(this.repository);

  @override
  Future<Result<int, Failure>> call(NoParams params) async {
    return await repository.getAssetCount();
  }
}
