//
// Copyright (c) 2024 Nathan Fiedler
//
import 'dart:typed_data';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/repositories/asset_repository.dart';
import 'package:tanuki/core/domain/usecases/replace_asset.dart';
import 'package:tanuki/core/error/failures.dart';

class MockAssetRepository extends Mock implements AssetRepository {}

void main() {
  late ReplaceAsset usecase;
  late MockAssetRepository mockAssetRepository;

  setUp(() {
    mockAssetRepository = MockAssetRepository();
    usecase = ReplaceAsset(mockAssetRepository);
  });

  setUpAll(() {
    // mocktail needs a fallback for any() that involves custom types
    Uint8List dummy = Uint8List(0);
    registerFallbackValue(dummy);
  });

  test(
    'should return asset identifier from the repository',
    () async {
      // arrange
      const Result<String, Failure> expected = Ok('asset123');
      when(() => mockAssetRepository.replaceAssetBytes(any(), any(), any()))
          .thenAnswer((_) async => const Ok('asset123'));
      // act
      final bytes = Uint8List(0);
      final params = Params(
        assetId: 'asset123',
        filename: 'happy.jpg',
        contents: bytes,
      );
      final result = await usecase(params);
      // assert
      expect(result, expected);
      expect(result.unwrap(), equals('asset123'));
      verify(() => mockAssetRepository.replaceAssetBytes(
          'asset123', 'happy.jpg', bytes));
      verifyNoMoreInteractions(mockAssetRepository);
    },
  );
}
