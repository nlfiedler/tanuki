//
// Copyright (c) 2022 Nathan Fiedler
//
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/get_all_tags.dart';
import 'package:tanuki/core/error/failures.dart';
import 'package:tanuki/features/browse/preso/bloc/all_tags_bloc.dart';

class MockEntityRepository extends Mock implements EntityRepository {}

void main() {
  late MockEntityRepository mockEntityRepository;
  late GetAllTags usecase;

  group('normal cases', () {
    final tags = [
      Tag(label: 'kittens', count: 806),
      Tag(label: 'snakes', count: 269),
      Tag(label: 'birds', count: 23),
    ];
    setUp(() {
      mockEntityRepository = MockEntityRepository();
      usecase = GetAllTags(mockEntityRepository);
      when(() => mockEntityRepository.getAllTags())
          .thenAnswer((_) async => Ok(tags));
    });

    blocTest(
      'emits [] when nothing is added',
      build: () => AllTagsBloc(usecase: usecase),
      expect: () => [],
    );

    blocTest(
      'emits [Loading, Loaded] when LoadAllTags is added',
      build: () => AllTagsBloc(usecase: usecase),
      act: (AllTagsBloc bloc) => bloc.add(LoadAllTags()),
      expect: () => [Loading(), Loaded(tags: tags)],
    );
  });

  group('error cases', () {
    setUp(() {
      mockEntityRepository = MockEntityRepository();
      usecase = GetAllTags(mockEntityRepository);
      when(() => mockEntityRepository.getAllTags())
          .thenAnswer((_) async => Err(ServerFailure('oh no!')));
    });

    blocTest(
      'emits [Loading, Error] when LoadAllTags is added',
      build: () => AllTagsBloc(usecase: usecase),
      act: (AllTagsBloc bloc) => bloc.add(LoadAllTags()),
      expect: () => [Loading(), Error(message: 'ServerFailure(oh no!)')],
    );
  });
}
