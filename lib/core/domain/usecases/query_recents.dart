//
// Copyright (c) 2023 Nathan Fiedler
//
import 'package:equatable/equatable.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';
import 'package:tanuki/core/error/failures.dart';

class QueryRecents implements UseCase<QueryResults, Params> {
  final EntityRepository repository;

  QueryRecents(this.repository);

  @override
  Future<Result<QueryResults, Failure>> call(Params params) async {
    return await repository.queryRecents(
      params.since,
      params.count,
      params.offset,
    );
  }
}

class Params extends Equatable {
  final Option<DateTime> since;
  final Option<int> count;
  final Option<int> offset;

  const Params({
    required this.since,
    required this.count,
    required this.offset,
  });

  @override
  List<Object> get props => [since, count, offset];
}
