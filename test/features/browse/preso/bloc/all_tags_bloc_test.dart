//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/get_all_tags.dart';
import 'package:tanuki/core/error/failures.dart';
import 'package:tanuki/features/browse/preso/bloc/all_tags_bloc.dart';

class MockConfigurationRepository extends Mock implements EntityRepository {}

void main() {
  MockConfigurationRepository mockConfigurationRepository;
  GetAllTags usecase;

  group('normal cases', () {
    final tags = [
      Tag(label: 'kittens', count: 806),
      Tag(label: 'snakes', count: 269),
      Tag(label: 'birds', count: 23),
    ];
    setUp(() {
      mockConfigurationRepository = MockConfigurationRepository();
      usecase = GetAllTags(mockConfigurationRepository);
      when(mockConfigurationRepository.getAllTags())
          .thenAnswer((_) async => Ok(tags));
    });

    blocTest(
      'emits [] when nothing is added',
      build: () => AllTagsBloc(usecase: usecase),
      expect: [],
    );

    blocTest(
      'emits [Loading, Loaded] when LoadAllTags is added',
      build: () => AllTagsBloc(usecase: usecase),
      act: (bloc) => bloc.add(LoadAllTags()),
      expect: [Loading(), Loaded(tags: tags)],
    );
  });

  group('error cases', () {
    setUp(() {
      mockConfigurationRepository = MockConfigurationRepository();
      usecase = GetAllTags(mockConfigurationRepository);
      when(mockConfigurationRepository.getAllTags())
          .thenAnswer((_) async => Err(ServerFailure('oh no!')));
    });

    blocTest(
      'emits [Loading, Error] when LoadAllTags is added',
      build: () => AllTagsBloc(usecase: usecase),
      act: (bloc) => bloc.add(LoadAllTags()),
      expect: [Loading(), Error(message: 'ServerFailure(oh no!)')],
    );
  });
}
