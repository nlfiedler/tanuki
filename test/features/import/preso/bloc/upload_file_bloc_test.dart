//
// Copyright (c) 2023 Nathan Fiedler
//
import 'dart:typed_data';
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/repositories/asset_repository.dart';
import 'package:tanuki/core/domain/usecases/upload_asset.dart';
import 'package:tanuki/core/error/failures.dart';
import 'package:tanuki/features/import/preso/bloc/upload_file_bloc.dart';

class MockAssetRepository extends Mock implements AssetRepository {}

void main() {
  late MockAssetRepository mockAssetRepository;
  late UploadAsset usecase;

  const tAssetId = 'MjAyMC8wNS8yNC8x-mini-N5emVhamE4ajZuLmpwZw==';

  setUpAll(() {
    // mocktail needs a fallback for any() that involves custom types
    Uint8List dummy = Uint8List(0);
    registerFallbackValue(dummy);
  });

  group('normal cases', () {
    setUp(() {
      mockAssetRepository = MockAssetRepository();
      usecase = UploadAsset(mockAssetRepository);
      when(() => mockAssetRepository.uploadAssetBytes(any(), any()))
          .thenAnswer((_) async => Ok(tAssetId));
    });

    blocTest(
      'emits [] when nothing is added',
      build: () => UploadFileBloc(usecase: usecase),
      expect: () => [],
    );

    blocTest(
      'emits [Uploading, Finished] when uploading/upload are added',
      build: () => UploadFileBloc(usecase: usecase),
      act: (UploadFileBloc bloc) {
        bloc.add(StartUploading(files: const ['foo']));
        bloc.add(UploadFile(filename: 'foo', contents: Uint8List(0)));
      },
      expect: () => [
        Uploading(pending: const [], current: 'foo'),
        Finished(skipped: const []),
      ],
    );

    blocTest(
      'emits [Uploading, Finished] when uploading/skip are added',
      build: () => UploadFileBloc(usecase: usecase),
      act: (UploadFileBloc bloc) {
        bloc.add(StartUploading(files: const ['foo']));
        bloc.add(SkipCurrent());
      },
      expect: () => [
        Uploading(pending: const [], current: 'foo'),
        Finished(skipped: const ['foo']),
      ],
    );

    blocTest(
      'emits [Uploading(x2), Finished] when multiple files are uploaded',
      build: () => UploadFileBloc(usecase: usecase),
      act: (UploadFileBloc bloc) {
        bloc.add(StartUploading(files: const ['foo', 'bar']));
        bloc.add(SkipCurrent());
        bloc.add(UploadFile(filename: 'foo', contents: Uint8List(0)));
      },
      expect: () => [
        Uploading(pending: const ['foo'], current: 'bar'),
        Uploading(pending: const [], current: 'foo', uploaded: 1),
        Finished(skipped: const ['bar']),
      ],
    );
  });

  group('error cases', () {
    setUp(() {
      mockAssetRepository = MockAssetRepository();
      usecase = UploadAsset(mockAssetRepository);
      when(() => mockAssetRepository.uploadAssetBytes(any(), any()))
          .thenAnswer((_) async => Err(ServerFailure('oh no!')));
    });

    blocTest(
      'emits [Uploading, Error] when repository returns an error',
      build: () => UploadFileBloc(usecase: usecase),
      act: (UploadFileBloc bloc) {
        bloc.add(StartUploading(files: const ['foo']));
        bloc.add(UploadFile(filename: 'foo', contents: Uint8List(0)));
      },
      expect: () => [
        Uploading(pending: const [], current: 'foo'),
        Error(message: 'ServerFailure(oh no!)'),
      ],
    );
  });
}
