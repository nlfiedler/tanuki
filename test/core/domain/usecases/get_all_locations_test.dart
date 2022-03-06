//
// Copyright (c) 2022 Nathan Fiedler
//
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/get_all_locations.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';
import 'package:tanuki/core/error/failures.dart';

class MockEntityRepository extends Mock implements EntityRepository {}

void main() {
  late GetAllLocations usecase;
  late MockEntityRepository mockEntityRepository;

  final locations = [
    Location(label: 'tokyo', count: 806),
    Location(label: 'paris', count: 269),
    Location(label: 'london', count: 23),
  ];

  setUp(() {
    mockEntityRepository = MockEntityRepository();
    usecase = GetAllLocations(mockEntityRepository);
  });

  test(
    'should get the list of locations from the repository',
    () async {
      // arrange
      final Result<List<Location>, Failure> expected = Ok(locations);
      when(() => mockEntityRepository.getAllLocations())
          .thenAnswer((_) async => Ok(locations));
      // act
      final result = await usecase(NoParams());
      // assert
      expect(result, expected);
      expect(result.unwrap()[0].label, 'london');
      expect(result.unwrap()[1].label, 'paris');
      expect(result.unwrap()[2].label, 'tokyo');
      verify(() => mockEntityRepository.getAllLocations());
      verifyNoMoreInteractions(mockEntityRepository);
    },
  );
}
