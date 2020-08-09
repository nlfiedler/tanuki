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

class MockConfigurationRepository extends Mock implements EntityRepository {}

void main() {
  GetAllLocations usecase;
  MockConfigurationRepository mockConfigurationRepository;

  final locations = [
    Location(label: 'tokyo', count: 806),
    Location(label: 'paris', count: 269),
    Location(label: 'london', count: 23),
  ];

  setUp(() {
    mockConfigurationRepository = MockConfigurationRepository();
    usecase = GetAllLocations(mockConfigurationRepository);
  });

  test(
    'should get the configuration from the repository',
    () async {
      // arrange
      when(mockConfigurationRepository.getAllLocations())
          .thenAnswer((_) async => Ok(locations));
      // act
      final result = await usecase(NoParams());
      // assert
      expect(result, Ok(locations));
      verify(mockConfigurationRepository.getAllLocations());
      verifyNoMoreInteractions(mockConfigurationRepository);
    },
  );
}
