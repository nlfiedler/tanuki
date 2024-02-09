//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/get_all_locations.dart';

class MockEntityRepository extends Mock implements EntityRepository {}

void main() {
  late GetAllLocations usecase;
  late MockEntityRepository mockEntityRepository;

  const locations = [
    Location(label: 'tokyo', count: 806),
    Location(label: 'paris', count: 269),
    Location(label: 'london', count: 23),
  ];

  setUp(() {
    mockEntityRepository = MockEntityRepository();
    usecase = GetAllLocations(mockEntityRepository);
  });

  test(
    'should get the list of all locations from the repository',
    () async {
      // arrange
      when(() => mockEntityRepository.getAllLocations(false))
          .thenAnswer((_) async => Ok(List.from(locations)));
      // act
      final result = await usecase(const Params(raw: false));
      // assert
      expect(result.unwrap().length, 3);
      expect(result.unwrap()[0].label, 'london');
      expect(result.unwrap()[1].label, 'paris');
      expect(result.unwrap()[2].label, 'tokyo');
      verify(() => mockEntityRepository.getAllLocations(false));
      verifyNoMoreInteractions(mockEntityRepository);
    },
  );

  test(
    'should get the list of raw locations from the repository',
    () async {
      // arrange
      when(() => mockEntityRepository.getAllLocations(true))
          .thenAnswer((_) async => Ok(List.from(locations)));
      // act
      final result = await usecase(const Params(raw: true));
      // assert
      expect(result.unwrap().length, 3);
      expect(result.unwrap()[0].label, 'london');
      expect(result.unwrap()[1].label, 'paris');
      expect(result.unwrap()[2].label, 'tokyo');
      verify(() => mockEntityRepository.getAllLocations(true));
      verifyNoMoreInteractions(mockEntityRepository);
    },
  );
}
