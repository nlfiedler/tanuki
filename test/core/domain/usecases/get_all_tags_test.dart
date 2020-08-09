//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/get_all_tags.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';

class MockConfigurationRepository extends Mock implements EntityRepository {}

void main() {
  GetAllTags usecase;
  MockConfigurationRepository mockConfigurationRepository;

  final tags = [
    Tag(label: 'kittens', count: 806),
    Tag(label: 'snakes', count: 269),
    Tag(label: 'birds', count: 23),
  ];

  setUp(() {
    mockConfigurationRepository = MockConfigurationRepository();
    usecase = GetAllTags(mockConfigurationRepository);
  });

  test(
    'should get the configuration from the repository',
    () async {
      // arrange
      when(mockConfigurationRepository.getAllTags())
          .thenAnswer((_) async => Ok(tags));
      // act
      final result = await usecase(NoParams());
      // assert
      expect(result, Ok(tags));
      verify(mockConfigurationRepository.getAllTags());
      verifyNoMoreInteractions(mockConfigurationRepository);
    },
  );
}
