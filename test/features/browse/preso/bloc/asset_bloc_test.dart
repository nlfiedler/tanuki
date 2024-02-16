//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/get_asset.dart';
import 'package:tanuki/core/error/failures.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_bloc.dart';

class MockEntityRepository extends Mock implements EntityRepository {}

void main() {
  late MockEntityRepository mockEntityRepository;
  late GetAsset usecase;

  final tAsset = Asset(
    id: 'MjAyMC8wNS8yNC8x-mini-N5emVhamE4ajZuLmpwZw==',
    checksum: 'sha256-34641209e88f3a59b-mini-2dfdcb00f8a533ac80ba',
    filename: 'catmouse_1280p.jpg',
    filesize: 160852,
    datetime: DateTime.utc(2020, 5, 24, 18, 02, 15),
    mimetype: 'image/jpeg',
    tags: const ['cat', 'mouse'],
    userdate: const None(),
    caption: const Some('#cat @outdoors #mouse'),
    location: const Some(AssetLocation(
      label: Some('outdoors'),
      city: None(),
      region: None(),
    )),
  );

  group('normal cases', () {
    setUp(() {
      mockEntityRepository = MockEntityRepository();
      usecase = GetAsset(mockEntityRepository);
      when(() => mockEntityRepository.getAsset(any()))
          .thenAnswer((_) async => Ok(tAsset));
    });

    blocTest(
      'emits [] when nothing is added',
      build: () => AssetBloc(usecase: usecase),
      expect: () => [],
    );

    blocTest(
      'emits [Loading, Loaded] when LoadAllDataSets is added',
      build: () => AssetBloc(usecase: usecase),
      act: (AssetBloc bloc) => bloc.add(LoadAsset(id: 'cafebabe')),
      expect: () => [Loading(), Loaded(asset: tAsset)],
    );
  });

  group('error cases', () {
    setUp(() {
      mockEntityRepository = MockEntityRepository();
      usecase = GetAsset(mockEntityRepository);
      when(() => mockEntityRepository.getAsset(any()))
          .thenAnswer((_) async => const Err(ServerFailure('oh no!')));
    });

    blocTest(
      'emits [Loading, Error] when LoadAllDataSets is added',
      build: () => AssetBloc(usecase: usecase),
      act: (AssetBloc bloc) => bloc.add(LoadAsset(id: 'cafebabe')),
      expect: () => [Loading(), Error(message: 'ServerFailure(oh no!)')],
    );
  });
}
