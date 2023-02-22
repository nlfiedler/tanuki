//
// Copyright (c) 2023 Nathan Fiedler
//
import 'package:equatable/equatable.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/input.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';
import 'package:tanuki/core/error/failures.dart';

class BulkUpdate implements UseCase<int, Params> {
  final EntityRepository repository;

  BulkUpdate(this.repository);

  @override
  Future<Result<int, Failure>> call(Params params) async {
    return await repository.bulkUpdate(params.assets);
  }
}

class Params extends Equatable {
  final List<AssetInputId> assets;

  const Params({
    required this.assets,
  });

  @override
  List<Object> get props => [assets];
}
