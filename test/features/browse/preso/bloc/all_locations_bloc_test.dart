//
// Copyright (c) 2022 Nathan Fiedler
//
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/get_all_locations.dart';
import 'package:tanuki/core/error/failures.dart';
import 'package:tanuki/features/browse/preso/bloc/all_locations_bloc.dart';

class MockEntityRepository extends Mock implements EntityRepository {}

void main() {
  late MockEntityRepository mockEntityRepository;
  late GetAllLocations usecase;

  group('normal cases', () {
    final locations = [
      Location(label: 'tokyo', count: 806),
      Location(label: 'paris', count: 269),
      Location(label: 'london', count: 23),
    ];
    setUp(() {
      mockEntityRepository = MockEntityRepository();
      usecase = GetAllLocations(mockEntityRepository);
      when(() => mockEntityRepository.getAllLocations())
          .thenAnswer((_) async => Ok(locations));
    });

    blocTest(
      'emits [] when nothing is added',
      build: () => AllLocationsBloc(usecase: usecase),
      expect: () => [],
    );

    blocTest(
      'emits [Loading, Loaded] when LoadAllLocations is added',
      build: () => AllLocationsBloc(usecase: usecase),
      act: (AllLocationsBloc bloc) => bloc.add(LoadAllLocations()),
      expect: () => [Loading(), Loaded(locations: locations)],
    );
  });

  group('error cases', () {
    setUp(() {
      mockEntityRepository = MockEntityRepository();
      usecase = GetAllLocations(mockEntityRepository);
      when(() => mockEntityRepository.getAllLocations())
          .thenAnswer((_) async => Err(ServerFailure('oh no!')));
    });

    blocTest(
      'emits [Loading, Error] when LoadAllLocations is added',
      build: () => AllLocationsBloc(usecase: usecase),
      act: (AllLocationsBloc bloc) => bloc.add(LoadAllLocations()),
      expect: () => [Loading(), Error(message: 'ServerFailure(oh no!)')],
    );
  });
}
