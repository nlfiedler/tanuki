//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:typed_data';
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/repositories/asset_repository.dart';
import 'package:tanuki/core/domain/usecases/upload_asset.dart';
import 'package:tanuki/core/error/failures.dart';
import 'package:tanuki/features/import/preso/bloc/upload_file_bloc.dart';

class MockAssetRepository extends Mock implements AssetRepository {}

void main() {
  MockAssetRepository mockAssetRepository;
  UploadAsset usecase;

  final tAssetId = 'MjAyMC8wNS8yNC8x-mini-N5emVhamE4ajZuLmpwZw==';

  group('normal cases', () {
    setUp(() {
      mockAssetRepository = MockAssetRepository();
      usecase = UploadAsset(mockAssetRepository);
      when(mockAssetRepository.uploadAssetBytes(any, any))
          .thenAnswer((_) async => Ok(tAssetId));
    });

    blocTest(
      'emits [] when nothing is added',
      build: () => UploadFileBloc(usecase: usecase),
      expect: [],
    );

    blocTest(
      'emits [Uploading, Finished] when uploading/upload are added',
      build: () => UploadFileBloc<String>(usecase: usecase),
      act: (bloc) {
        bloc.add(StartUploading<String>(files: ['foo']));
        bloc.add(UploadFile(filename: 'foo', contents: Uint8List(0)));
      },
      expect: [
        Uploading<String>(pending: [], current: 'foo'),
        Finished<String>(skipped: []),
      ],
    );

    blocTest(
      'emits [Uploading, Finished] when uploading/skip are added',
      build: () => UploadFileBloc<String>(usecase: usecase),
      act: (bloc) {
        bloc.add(StartUploading<String>(files: ['foo']));
        bloc.add(SkipCurrent());
      },
      expect: [
        Uploading<String>(pending: [], current: 'foo'),
        Finished<String>(skipped: ['foo']),
      ],
    );

    blocTest(
      'emits [Uploading(x2), Finished] when multiple files are uploaded',
      build: () => UploadFileBloc<String>(usecase: usecase),
      act: (bloc) {
        bloc.add(StartUploading<String>(files: ['foo', 'bar']));
        bloc.add(SkipCurrent());
        bloc.add(UploadFile(filename: 'foo', contents: Uint8List(0)));
      },
      expect: [
        Uploading<String>(pending: ['foo'], current: 'bar'),
        Uploading<String>(pending: [], current: 'foo'),
        Finished<String>(skipped: ['bar']),
      ],
    );
  });

  group('error cases', () {
    setUp(() {
      mockAssetRepository = MockAssetRepository();
      usecase = UploadAsset(mockAssetRepository);
      when(mockAssetRepository.uploadAssetBytes(any, any))
          .thenAnswer((_) async => Err(ServerFailure('oh no!')));
    });

    blocTest(
      'emits [Uploading, Error] when repository returns an error',
      build: () => UploadFileBloc<String>(usecase: usecase),
      act: (bloc) {
        bloc.add(StartUploading<String>(files: ['foo']));
        bloc.add(UploadFile(filename: 'foo', contents: Uint8List(0)));
      },
      expect: [
        Uploading<String>(pending: [], current: 'foo'),
        Error(message: 'ServerFailure(oh no!)'),
      ],
    );
  });
}
