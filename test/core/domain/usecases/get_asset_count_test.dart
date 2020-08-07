//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/get_asset_count.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';

class MockConfigurationRepository extends Mock implements EntityRepository {}

void main() {
  GetAssetCount usecase;
  MockConfigurationRepository mockConfigurationRepository;

  setUp(() {
    mockConfigurationRepository = MockConfigurationRepository();
    usecase = GetAssetCount(mockConfigurationRepository);
  });

  test(
    'should get the configuration from the repository',
    () async {
      // arrange
      when(mockConfigurationRepository.getAssetCount())
          .thenAnswer((_) async => Ok(9413));
      // act
      final result = await usecase(NoParams());
      // assert
      expect(result, Ok(9413));
      verify(mockConfigurationRepository.getAssetCount());
      verifyNoMoreInteractions(mockConfigurationRepository);
    },
  );
}
