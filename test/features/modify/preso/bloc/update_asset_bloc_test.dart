//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/domain/entities/input.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/update_asset.dart';
import 'package:tanuki/core/error/failures.dart';
import 'package:tanuki/features/modify/preso/bloc/update_asset_bloc.dart';

class MockEntityRepository extends Mock implements EntityRepository {}

void main() {
  late MockEntityRepository mockAssetRepository;
  late UpdateAsset usecase;

  final inputId = AssetInputId(
    id: 'asset123',
    input: AssetInput(
      tags: const ['clowns', 'snakes'],
      caption: const Some('#snakes and #clowns are in my @batcave'),
      location: Some(AssetLocation.from('batcave')),
      datetime: Some(DateTime.utc(2003, 8, 30)),
      mediaType: const Some('image/jpeg'),
      filename: const Some('img_1234.jpg'),
    ),
  );

  final expected = Asset(
    id: 'MjAyMC8wNS8yNC8x-mini-N5emVhamE4ajZuLmpwZw==',
    checksum: 'sha256-34641209e88f3a59b-mini-2dfdcb00f8a533ac80ba',
    filename: 'catmouse_1280p.jpg',
    filepath: 'leading/path/image.jpg',
    filesize: 160852,
    datetime: DateTime.utc(2020, 5, 24, 18, 02, 15),
    mediaType: 'image/jpeg',
    tags: const ['cat', 'mouse'],
    userdate: const None(),
    caption: const Some('#cat @outdoors #mouse'),
    location: const Some(AssetLocation(
      label: Some('outdoors'),
      city: None(),
      region: None(),
    )),
  );

  setUpAll(() {
    // mocktail needs a fallback for any() that involves custom types
    registerFallbackValue(inputId);
  });

  group('normal cases', () {
    setUp(() {
      mockAssetRepository = MockEntityRepository();
      usecase = UpdateAsset(mockAssetRepository);
      when(() => mockAssetRepository.updateAsset(any()))
          .thenAnswer((_) async => Ok(expected));
    });

    blocTest(
      'emits [] when nothing is added',
      build: () => UpdateAssetBloc(usecase: usecase),
      expect: () => [],
    );

    blocTest(
      'emits [Processing, Finished] when ProcessUploads is added',
      build: () => UpdateAssetBloc(usecase: usecase),
      act: (UpdateAssetBloc bloc) => bloc.add(SubmitUpdate(input: inputId)),
      expect: () => [
        Processing(),
        Finished(asset: expected),
      ],
    );
  });

  group('error cases', () {
    setUp(() {
      mockAssetRepository = MockEntityRepository();
      usecase = UpdateAsset(mockAssetRepository);
      when(() => mockAssetRepository.updateAsset(any()))
          .thenAnswer((_) async => const Err(ServerFailure('oh no!')));
    });

    blocTest(
      'emits [Uploading, Error] when repository returns an error',
      build: () => UpdateAssetBloc(usecase: usecase),
      act: (UpdateAssetBloc bloc) => bloc.add(SubmitUpdate(input: inputId)),
      expect: () => [
        Processing(),
        Error(message: 'ServerFailure(oh no!)'),
      ],
    );
  });
}
