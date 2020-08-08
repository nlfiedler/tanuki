//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/get_all_years.dart';
import 'package:tanuki/core/error/failures.dart';
import 'package:tanuki/features/browse/preso/bloc/all_years_bloc.dart';

class MockConfigurationRepository extends Mock implements EntityRepository {}

void main() {
  MockConfigurationRepository mockConfigurationRepository;
  GetAllYears usecase;

  group('normal cases', () {
    final years = [
      Year(label: '2019', count: 806),
      Year(label: '2009', count: 269),
      Year(label: '1999', count: 23),
    ];
    setUp(() {
      mockConfigurationRepository = MockConfigurationRepository();
      usecase = GetAllYears(mockConfigurationRepository);
      when(mockConfigurationRepository.getAllYears())
          .thenAnswer((_) async => Ok(years));
    });

    blocTest(
      'emits [] when nothing is added',
      build: () => AllYearsBloc(usecase: usecase),
      expect: [],
    );

    blocTest(
      'emits [Loading, Loaded] when LoadAllYears is added',
      build: () => AllYearsBloc(usecase: usecase),
      act: (bloc) => bloc.add(LoadAllYears()),
      expect: [Loading(), Loaded(years: years)],
    );
  });

  group('error cases', () {
    setUp(() {
      mockConfigurationRepository = MockConfigurationRepository();
      usecase = GetAllYears(mockConfigurationRepository);
      when(mockConfigurationRepository.getAllYears())
          .thenAnswer((_) async => Err(ServerFailure('oh no!')));
    });

    blocTest(
      'emits [Loading, Error] when LoadAllYears is added',
      build: () => AllYearsBloc(usecase: usecase),
      act: (bloc) => bloc.add(LoadAllYears()),
      expect: [Loading(), Error(message: 'ServerFailure(oh no!)')],
    );
  });
}
