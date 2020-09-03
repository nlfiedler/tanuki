//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/repositories/asset_repository.dart';
import 'package:tanuki/core/domain/usecases/ingest_assets.dart';
import 'package:tanuki/core/error/failures.dart';
import 'package:tanuki/features/upload/preso/bloc/ingest_assets_bloc.dart';

class MockAssetRepository extends Mock implements AssetRepository {}

void main() {
  MockAssetRepository mockAssetRepository;
  IngestAssets usecase;

  group('normal cases', () {
    setUp(() {
      mockAssetRepository = MockAssetRepository();
      usecase = IngestAssets(mockAssetRepository);
      when(mockAssetRepository.ingestAssets()).thenAnswer((_) async => Ok(101));
    });

    blocTest(
      'emits [] when nothing is added',
      build: () => IngestAssetsBloc(usecase: usecase),
      expect: [],
    );

    blocTest(
      'emits [Processing, Finished] when ProcessUploads is added',
      build: () => IngestAssetsBloc(usecase: usecase),
      act: (bloc) => bloc.add(ProcessUploads()),
      expect: [
        Processing(),
        Finished(count: 101),
      ],
    );
  });

  group('error cases', () {
    setUp(() {
      mockAssetRepository = MockAssetRepository();
      usecase = IngestAssets(mockAssetRepository);
      when(mockAssetRepository.ingestAssets())
          .thenAnswer((_) async => Err(ServerFailure('oh no!')));
    });

    blocTest(
      'emits [Uploading, Error] when repository returns an error',
      build: () => IngestAssetsBloc(usecase: usecase),
      act: (bloc) => bloc.add(ProcessUploads()),
      expect: [
        Processing(),
        Error(message: 'ServerFailure(oh no!)'),
      ],
    );
  });
}
