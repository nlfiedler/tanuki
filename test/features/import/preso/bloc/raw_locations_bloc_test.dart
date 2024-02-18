//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/get_asset_locations.dart';
import 'package:tanuki/core/error/failures.dart';
import 'package:tanuki/features/import/preso/bloc/raw_locations_bloc.dart';

class MockEntityRepository extends Mock implements EntityRepository {}

void main() {
  late MockEntityRepository mockEntityRepository;
  late GetAssetLocations usecase;

  group('normal cases', () {
    const incoming = [
      AssetLocation(label: None(), city: Some('Tokyo'), region: Some('Japan')),
      AssetLocation(
        label: Some('eiffel'),
        city: Some('Paris'),
        region: Some('France'),
      ),
      AssetLocation(label: None(), city: Some('London'), region: Some('UK')),
    ];
    // the incoming list will be served in sorted order
    const expected = [
      AssetLocation(label: None(), city: Some('Tokyo'), region: Some('Japan')),
      AssetLocation(
        label: Some('eiffel'),
        city: Some('Paris'),
        region: Some('France'),
      ),
      AssetLocation(label: None(), city: Some('London'), region: Some('UK')),
    ];
    setUp(() {
      mockEntityRepository = MockEntityRepository();
      usecase = GetAssetLocations(mockEntityRepository);
      when(() => mockEntityRepository.getAssetLocations())
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
      usecase = GetAssetLocations(mockEntityRepository);
      when(() => mockEntityRepository.getAssetLocations())
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
