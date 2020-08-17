//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/get_all_locations.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';

class MockEntityRepository extends Mock implements EntityRepository {}

void main() {
  GetAllLocations usecase;
  MockEntityRepository mockEntityRepository;

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
      when(mockEntityRepository.getAllLocations())
          .thenAnswer((_) async => Ok(locations));
      // act
      final result = await usecase(NoParams());
      // assert
      expect(result, Ok(locations));
      expect(result.unwrap()[0].label, 'london');
      expect(result.unwrap()[1].label, 'paris');
      expect(result.unwrap()[2].label, 'tokyo');
      verify(mockEntityRepository.getAllLocations());
      verifyNoMoreInteractions(mockEntityRepository);
    },
  );
}
