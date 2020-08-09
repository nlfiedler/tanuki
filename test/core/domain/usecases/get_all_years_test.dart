//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/get_all_years.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';

class MockConfigurationRepository extends Mock implements EntityRepository {}

void main() {
  GetAllYears usecase;
  MockConfigurationRepository mockConfigurationRepository;

  final years = [
    Year(label: '2019', count: 806),
    Year(label: '2009', count: 269),
    Year(label: '1999', count: 23),
  ];

  setUp(() {
    mockConfigurationRepository = MockConfigurationRepository();
    usecase = GetAllYears(mockConfigurationRepository);
  });

  test(
    'should get the configuration from the repository',
    () async {
      // arrange
      when(mockConfigurationRepository.getAllYears())
          .thenAnswer((_) async => Ok(years));
      // act
      final result = await usecase(NoParams());
      // assert
      expect(result, Ok(years));
      verify(mockConfigurationRepository.getAllYears());
      verifyNoMoreInteractions(mockConfigurationRepository);
    },
  );
}
