//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';
import 'package:tanuki/core/error/failures.dart';

class GetAllLocations implements UseCase<List<Location>, NoParams> {
  final EntityRepository repository;

  GetAllLocations(this.repository);

  @override
  Future<Result<List<Location>, Failure>> call(NoParams params) async {
    var result = await repository.getAllLocations();
    result = result.map((locations) {
      locations.sort();
      return locations;
    });
    return result;
  }
}
