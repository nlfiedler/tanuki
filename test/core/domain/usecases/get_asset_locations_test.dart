//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/get_asset_locations.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';

class MockEntityRepository extends Mock implements EntityRepository {}

void main() {
  late GetAssetLocations usecase;
  late MockEntityRepository mockEntityRepository;

  const locations = [
    AssetLocation(label: Some('tokyo'), city: None(), region: None()),
    AssetLocation(label: Some('paris'), city: None(), region: None()),
    AssetLocation(label: Some('london'), city: None(), region: None()),
  ];

  setUp(() {
    mockEntityRepository = MockEntityRepository();
    usecase = GetAssetLocations(mockEntityRepository);
  });

  test(
    'should get the list of all locations from the repository',
    () async {
      // arrange
      when(() => mockEntityRepository.getAssetLocations())
          .thenAnswer((_) async => Ok(List.from(locations)));
      // act
      final result = await usecase(NoParams());
      // assert
      expect(result.unwrap().length, 3);
      expect(result.unwrap()[0].label.unwrap(), 'tokyo');
      expect(result.unwrap()[1].label.unwrap(), 'paris');
      expect(result.unwrap()[2].label.unwrap(), 'london');
      verify(() => mockEntityRepository.getAssetLocations());
      verifyNoMoreInteractions(mockEntityRepository);
    },
  );

  test(
    'should get the list of raw locations from the repository',
    () async {
      // arrange
      when(() => mockEntityRepository.getAssetLocations())
          .thenAnswer((_) async => Ok(List.from(locations)));
      // act
      final result = await usecase(NoParams());
      // assert
      expect(result.unwrap().length, 3);
      expect(result.unwrap()[0].label.unwrap(), 'tokyo');
      expect(result.unwrap()[1].label.unwrap(), 'paris');
      expect(result.unwrap()[2].label.unwrap(), 'london');
      verify(() => mockEntityRepository.getAssetLocations());
      verifyNoMoreInteractions(mockEntityRepository);
    },
  );
}
