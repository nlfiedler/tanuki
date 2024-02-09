//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:equatable/equatable.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';
import 'package:tanuki/core/error/failures.dart';

class GetAllLocations implements UseCase<List<Location>, Params> {
  final EntityRepository repository;

  GetAllLocations(this.repository);

  @override
  Future<Result<List<Location>, Failure>> call(Params params) async {
    var result = await repository.getAllLocations(params.raw);
    result = result.map((locations) {
      locations.sort();
      return locations;
    });
    return result;
  }
}

class Params extends Equatable {
  final bool raw;

  const Params({
    required this.raw,
  });

  @override
  List<Object> get props => [raw];
}
