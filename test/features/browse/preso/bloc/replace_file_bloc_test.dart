//
// Copyright (c) 2024 Nathan Fiedler
//
import 'dart:typed_data';
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/repositories/asset_repository.dart';
import 'package:tanuki/core/domain/usecases/replace_asset.dart';
import 'package:tanuki/core/error/failures.dart';
import 'package:tanuki/features/browse/preso/bloc/replace_file_bloc.dart';

class MockAssetRepository extends Mock implements AssetRepository {}

void main() {
  late MockAssetRepository mockAssetRepository;
  late ReplaceAsset usecase;

  const tAssetId = 'MjAyMC8wNS8yNC8x-mini-N5emVhamE4ajZuLmpwZw==';

  setUpAll(() {
    // mocktail needs a fallback for any() that involves custom types
    Uint8List dummy = Uint8List(0);
    registerFallbackValue(dummy);
  });

  group('normal cases', () {
    setUp(() {
      mockAssetRepository = MockAssetRepository();
      usecase = ReplaceAsset(mockAssetRepository);
      when(() => mockAssetRepository.replaceAssetBytes(any(), any(), any()))
          .thenAnswer((_) async => const Ok(tAssetId));
    });

    blocTest(
      'emits [] when nothing is added',
      build: () => ReplaceFileBloc(usecase: usecase),
      expect: () => [],
    );

    blocTest(
      'emits [Uploading, Finished] when replaceing/replace are added',
      build: () => ReplaceFileBloc(usecase: usecase),
      act: (ReplaceFileBloc bloc) {
        bloc.add(StartUploading(file: 'filename.jpg'));
        bloc.add(ReplaceFile(
          assetId: 'asset123',
          filename: 'filename.jpg',
          contents: Uint8List(0),
        ));
      },
      expect: () => [
        Uploading(current: 'filename.jpg'),
        Finished(assetId: tAssetId),
      ],
    );

    blocTest(
      'emits [Uploading, Finished] when replaceing/skip are added',
      build: () => ReplaceFileBloc(usecase: usecase),
      act: (ReplaceFileBloc bloc) {
        bloc.add(StartUploading(file: 'filename.jpg'));
        bloc.add(SkipCurrent());
      },
      expect: () => [
        Uploading(current: 'filename.jpg'),
        Initial(),
      ],
    );
  });

  group('error cases', () {
    setUp(() {
      mockAssetRepository = MockAssetRepository();
      usecase = ReplaceAsset(mockAssetRepository);
      when(() => mockAssetRepository.replaceAssetBytes(any(), any(), any()))
          .thenAnswer((_) async => const Err(ServerFailure('oh no!')));
    });

    blocTest(
      'emits [Uploading, Error] when repository returns an error',
      build: () => ReplaceFileBloc(usecase: usecase),
      act: (ReplaceFileBloc bloc) {
        bloc.add(StartUploading(file: 'filename.jpg'));
        bloc.add(ReplaceFile(
          assetId: 'asset123',
          filename: 'filename.jpg',
          contents: Uint8List(0),
        ));
      },
      expect: () => [
        Uploading(current: 'filename.jpg'),
        Error(message: 'ServerFailure(oh no!)'),
      ],
    );
  });
}
