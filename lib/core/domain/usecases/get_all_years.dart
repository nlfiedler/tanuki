//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';
import 'package:tanuki/core/error/failures.dart';

class GetAllYears implements UseCase<List<Year>, NoParams> {
  final EntityRepository repository;

  GetAllYears(this.repository);

  @override
  Future<Result<List<Year>, Failure>> call(NoParams params) async {
    return await repository.getAllYears();
  }
}
