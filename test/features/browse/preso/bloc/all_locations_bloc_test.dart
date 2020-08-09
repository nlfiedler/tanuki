//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/get_all_locations.dart';
import 'package:tanuki/core/error/failures.dart';
import 'package:tanuki/features/browse/preso/bloc/all_locations_bloc.dart';

class MockConfigurationRepository extends Mock implements EntityRepository {}

void main() {
  MockConfigurationRepository mockConfigurationRepository;
  GetAllLocations usecase;

  group('normal cases', () {
    final locations = [
      Location(label: 'tokyo', count: 806),
      Location(label: 'paris', count: 269),
      Location(label: 'london', count: 23),
    ];
    setUp(() {
      mockConfigurationRepository = MockConfigurationRepository();
      usecase = GetAllLocations(mockConfigurationRepository);
      when(mockConfigurationRepository.getAllLocations())
          .thenAnswer((_) async => Ok(locations));
    });

    blocTest(
      'emits [] when nothing is added',
      build: () => AllLocationsBloc(usecase: usecase),
      expect: [],
    );

    blocTest(
      'emits [Loading, Loaded] when LoadAllLocations is added',
      build: () => AllLocationsBloc(usecase: usecase),
      act: (bloc) => bloc.add(LoadAllLocations()),
      expect: [Loading(), Loaded(locations: locations)],
    );
  });

  group('error cases', () {
    setUp(() {
      mockConfigurationRepository = MockConfigurationRepository();
      usecase = GetAllLocations(mockConfigurationRepository);
      when(mockConfigurationRepository.getAllLocations())
          .thenAnswer((_) async => Err(ServerFailure('oh no!')));
    });

    blocTest(
      'emits [Loading, Error] when LoadAllLocations is added',
      build: () => AllLocationsBloc(usecase: usecase),
      act: (bloc) => bloc.add(LoadAllLocations()),
      expect: [Loading(), Error(message: 'ServerFailure(oh no!)')],
    );
  });
}
