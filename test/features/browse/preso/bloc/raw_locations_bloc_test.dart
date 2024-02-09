//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/get_all_locations.dart';
import 'package:tanuki/core/error/failures.dart';
import 'package:tanuki/features/browse/preso/bloc/raw_locations_bloc.dart';

class MockEntityRepository extends Mock implements EntityRepository {}

void main() {
  late MockEntityRepository mockEntityRepository;
  late GetAllLocations usecase;

  group('normal cases', () {
    const incoming = [
      Location(label: 'tokyo', count: 806),
      Location(label: 'paris', count: 269),
      Location(label: 'london', count: 23),
    ];
    // the incoming list will be served in sorted order
    const expected = [
      Location(label: 'london', count: 23),
      Location(label: 'paris', count: 269),
      Location(label: 'tokyo', count: 806),
    ];
    setUp(() {
      mockEntityRepository = MockEntityRepository();
      usecase = GetAllLocations(mockEntityRepository);
      when(() => mockEntityRepository.getAllLocations(true))
          .thenAnswer((_) async => Ok(List.from(incoming)));
    });

    blocTest(
      'emits [] when nothing is added',
      build: () => RawLocationsBloc(usecase: usecase),
      expect: () => [],
    );

    blocTest(
      'emits [Loading, Loaded] when LoadRawLocations is added',
      build: () => RawLocationsBloc(usecase: usecase),
      act: (RawLocationsBloc bloc) => bloc.add(LoadRawLocations()),
      expect: () => [Loading(), Loaded(locations: expected)],
    );
  });

  group('error cases', () {
    setUp(() {
      mockEntityRepository = MockEntityRepository();
      usecase = GetAllLocations(mockEntityRepository);
      when(() => mockEntityRepository.getAllLocations(true))
          .thenAnswer((_) async => const Err(ServerFailure('oh no!')));
    });

    blocTest(
      'emits [Loading, Error] when LoadAllLocations is added',
      build: () => RawLocationsBloc(usecase: usecase),
      act: (RawLocationsBloc bloc) => bloc.add(LoadRawLocations()),
      expect: () => [Loading(), Error(message: 'ServerFailure(oh no!)')],
    );
  });
}
