//
// Copyright (c) 2023 Nathan Fiedler
//
import 'package:equatable/equatable.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';
import 'package:tanuki/core/error/failures.dart';

class QueryAssets implements UseCase<QueryResults, Params> {
  final EntityRepository repository;

  QueryAssets(this.repository);

  @override
  Future<Result<QueryResults, Failure>> call(Params params) async {
    return await repository.queryAssets(
      params.params,
      params.count,
      params.offset,
    );
  }
}

class Params extends Equatable {
  final SearchParams params;
  final int count;
  final int offset;

  const Params({
    required this.params,
    required this.count,
    required this.offset,
  });

  @override
  List<Object> get props => [params, count, offset];
}
