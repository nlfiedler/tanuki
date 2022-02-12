//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:typed_data';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:mockito/annotations.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/repositories/asset_repository.dart';
import 'package:tanuki/core/domain/usecases/upload_asset.dart';
import 'package:tanuki/core/error/failures.dart';
import './upload_asset_test.mocks.dart';

@GenerateMocks([AssetRepository])
void main() {
  late UploadAsset usecase;
  late MockAssetRepository mockAssetRepository;

  setUp(() {
    mockAssetRepository = MockAssetRepository();
    usecase = UploadAsset(mockAssetRepository);
  });

  test(
    'should return asset identifier from the repository',
    () async {
      // arrange
      final Result<String, Failure> expected = Ok('asset123');
      when(mockAssetRepository.uploadAssetBytes(any, any))
          .thenAnswer((_) async => Ok('asset123'));
      // act
      final bytes = Uint8List(0);
      final params = Params(filename: 'happy.jpg', contents: bytes);
      final result = await usecase(params);
      // assert
      expect(result, expected);
      expect(result.unwrap(), equals('asset123'));
      verify(mockAssetRepository.uploadAssetBytes('happy.jpg', bytes));
      verifyNoMoreInteractions(mockAssetRepository);
    },
  );
}
