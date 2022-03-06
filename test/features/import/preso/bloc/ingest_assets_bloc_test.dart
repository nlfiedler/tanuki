//
// Copyright (c) 2022 Nathan Fiedler
//
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/repositories/asset_repository.dart';
import 'package:tanuki/core/domain/usecases/ingest_assets.dart';
import 'package:tanuki/core/error/failures.dart';
import 'package:tanuki/features/import/preso/bloc/ingest_assets_bloc.dart';

class MockAssetRepository extends Mock implements AssetRepository {}

void main() {
  late MockAssetRepository mockAssetRepository;
  late IngestAssets usecase;

  group('normal cases', () {
    setUp(() {
      mockAssetRepository = MockAssetRepository();
      usecase = IngestAssets(mockAssetRepository);
      when(() => mockAssetRepository.ingestAssets())
          .thenAnswer((_) async => Ok(101));
    });

    blocTest(
      'emits [] when nothing is added',
      build: () => IngestAssetsBloc(usecase: usecase),
      expect: () => [],
    );

    blocTest(
      'emits [Processing, Finished] when ProcessUploads is added',
      build: () => IngestAssetsBloc(usecase: usecase),
      act: (IngestAssetsBloc bloc) => bloc.add(ProcessUploads()),
      expect: () => [
        Processing(),
        Finished(count: 101),
      ],
    );
  });

  group('error cases', () {
    setUp(() {
      mockAssetRepository = MockAssetRepository();
      usecase = IngestAssets(mockAssetRepository);
      when(() => mockAssetRepository.ingestAssets())
          .thenAnswer((_) async => Err(ServerFailure('oh no!')));
    });

    blocTest(
      'emits [Uploading, Error] when repository returns an error',
      build: () => IngestAssetsBloc(usecase: usecase),
      act: (IngestAssetsBloc bloc) => bloc.add(ProcessUploads()),
      expect: () => [
        Processing(),
        Error(message: 'ServerFailure(oh no!)'),
      ],
    );
  });
}
