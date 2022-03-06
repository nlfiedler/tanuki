//
// Copyright (c) 2022 Nathan Fiedler
//
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/get_all_tags.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';
import 'package:tanuki/core/error/failures.dart';

class MockEntityRepository extends Mock implements EntityRepository {}

void main() {
  late GetAllTags usecase;
  late MockEntityRepository mockEntityRepository;

  final tags = [
    Tag(label: 'kittens', count: 806),
    Tag(label: 'snakes', count: 269),
    Tag(label: 'birds', count: 23),
  ];

  setUp(() {
    mockEntityRepository = MockEntityRepository();
    usecase = GetAllTags(mockEntityRepository);
  });

  test(
    'should get the list of tags from the repository',
    () async {
      // arrange
      final Result<List<Tag>, Failure> expected = Ok(tags);
      when(() => mockEntityRepository.getAllTags())
          .thenAnswer((_) async => Ok(tags));
      // act
      final result = await usecase(NoParams());
      // assert
      expect(result, expected);
      expect(result.unwrap()[0].label, 'birds');
      expect(result.unwrap()[1].label, 'kittens');
      expect(result.unwrap()[2].label, 'snakes');
      verify(() => mockEntityRepository.getAllTags());
      verifyNoMoreInteractions(mockEntityRepository);
    },
  );
}
