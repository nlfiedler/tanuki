//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:mockito/annotations.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/repositories/asset_repository.dart';
import 'package:tanuki/core/domain/usecases/ingest_assets.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';
import 'package:tanuki/core/error/failures.dart';
import './ingest_assets_test.mocks.dart';

@GenerateMocks([AssetRepository])
void main() {
  late IngestAssets usecase;
  late MockAssetRepository mockAssetRepository;

  setUp(() {
    mockAssetRepository = MockAssetRepository();
    usecase = IngestAssets(mockAssetRepository);
  });

  test(
    'should query assets from the repository',
    () async {
      // arrange
      final Result<int, Failure> expected = Ok(101);
      when(mockAssetRepository.ingestAssets()).thenAnswer((_) async => Ok(101));
      // act
      final params = NoParams();
      final result = await usecase(params);
      // assert
      expect(result, expected);
      expect(result.unwrap(), equals(101));
      verify(mockAssetRepository.ingestAssets());
      verifyNoMoreInteractions(mockAssetRepository);
    },
  );
}
