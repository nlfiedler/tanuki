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

class MockEntityRepository extends Mock implements EntityRepository {}

void main() {
  GetAllYears usecase;
  MockEntityRepository mockEntityRepository;

  final years = [
    Year(label: '2019', count: 806),
    Year(label: '2009', count: 269),
    Year(label: '1999', count: 23),
  ];

  setUp(() {
    mockEntityRepository = MockEntityRepository();
    usecase = GetAllYears(mockEntityRepository);
  });

  test(
    'should get the list of years from the repository',
    () async {
      // arrange
      when(mockEntityRepository.getAllYears())
          .thenAnswer((_) async => Ok(years));
      // act
      final result = await usecase(NoParams());
      // assert
      expect(result, Ok(years));
      expect(result.unwrap()[0].label, '1999');
      expect(result.unwrap()[1].label, '2009');
      expect(result.unwrap()[2].label, '2019');
      verify(mockEntityRepository.getAllYears());
      verifyNoMoreInteractions(mockEntityRepository);
    },
  );
}
