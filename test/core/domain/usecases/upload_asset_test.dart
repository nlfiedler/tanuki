//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:typed_data';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/repositories/asset_repository.dart';
import 'package:tanuki/core/domain/usecases/upload_asset.dart';

class MockAssetRepository extends Mock implements AssetRepository {}

void main() {
  UploadAsset usecase;
  MockAssetRepository mockAssetRepository;

  setUp(() {
    mockAssetRepository = MockAssetRepository();
    usecase = UploadAsset(mockAssetRepository);
  });

  test(
    'should return asset identifier from the repository',
    () async {
      // arrange
      final expected = 'asset123';
      when(mockAssetRepository.uploadAssetBytes(any, any))
          .thenAnswer((_) async => Ok(expected));
      // act
      final bytes = Uint8List(0);
      final params = Params(filename: 'happy.jpg', contents: bytes);
      final result = await usecase(params);
      // assert
      expect(result, Ok(expected));
      expect(result.unwrap(), equals(expected));
      verify(mockAssetRepository.uploadAssetBytes('happy.jpg', bytes));
      verifyNoMoreInteractions(mockAssetRepository);
    },
  );
}
