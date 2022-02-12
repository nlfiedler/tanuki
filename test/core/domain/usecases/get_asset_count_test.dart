//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:mockito/annotations.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/get_asset_count.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';
import 'package:tanuki/core/error/failures.dart';
import './get_asset_count_test.mocks.dart';

@GenerateMocks([EntityRepository])
void main() {
  late GetAssetCount usecase;
  late MockEntityRepository mockEntityRepository;

  setUp(() {
    mockEntityRepository = MockEntityRepository();
    usecase = GetAssetCount(mockEntityRepository);
  });

  test(
    'should get the configuration from the repository',
    () async {
      // arrange
      final Ok<int, Failure> expected = Ok(9413);
      when(mockEntityRepository.getAssetCount())
          .thenAnswer((_) async => Ok(9413));
      // act
      final result = await usecase(NoParams());
      // assert
      expect(result, expected);
      verify(mockEntityRepository.getAssetCount());
      verifyNoMoreInteractions(mockEntityRepository);
    },
  );
}
