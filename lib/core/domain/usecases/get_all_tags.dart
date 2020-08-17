//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';
import 'package:tanuki/core/error/failures.dart';

class GetAllTags implements UseCase<List<Tag>, NoParams> {
  final EntityRepository repository;

  GetAllTags(this.repository);

  @override
  Future<Result<List<Tag>, Failure>> call(NoParams params) async {
    var result = await repository.getAllTags();
    result = result.map((tags) {
      tags.sort();
      return tags;
    });
    return result;
  }
}
