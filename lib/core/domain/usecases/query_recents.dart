//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:equatable/equatable.dart';
import 'package:meta/meta.dart';
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
    return await repository.queryRecents(params.since);
  }
}

class Params extends Equatable {
  final DateTime since;

  Params({
    @required this.since,
  });

  @override
  List<Object> get props => [since];
}
