//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:mockito/annotations.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/get_asset_count.dart';
import 'package:tanuki/core/error/failures.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_count_bloc.dart';
import './asset_count_bloc_test.mocks.dart';

@GenerateMocks([EntityRepository])
void main() {
  late MockEntityRepository mockEntityRepository;
  late GetAssetCount usecase;

  group('normal cases', () {
    setUp(() {
      mockEntityRepository = MockEntityRepository();
      usecase = GetAssetCount(mockEntityRepository);
      when(mockEntityRepository.getAssetCount())
          .thenAnswer((_) async => Ok(9413));
    });

    blocTest(
      'emits [] when nothing is added',
      build: () => AssetCountBloc(usecase: usecase),
      expect: () => [],
    );

    blocTest(
      'emits [Loading, Loaded] when LoadAssetCount is added',
      build: () => AssetCountBloc(usecase: usecase),
      act: (AssetCountBloc bloc) => bloc.add(LoadAssetCount()),
      expect: () => [Loading(), Loaded(count: 9413)],
    );
  });

  group('error cases', () {
    setUp(() {
      mockEntityRepository = MockEntityRepository();
      usecase = GetAssetCount(mockEntityRepository);
      when(mockEntityRepository.getAssetCount())
          .thenAnswer((_) async => Err(ServerFailure('oh no!')));
    });

    blocTest(
      'emits [Loading, Error] when LoadAssetCount is added',
      build: () => AssetCountBloc(usecase: usecase),
      act: (AssetCountBloc bloc) => bloc.add(LoadAssetCount()),
      expect: () => [Loading(), Error(message: 'ServerFailure(oh no!)')],
    );
  });
}
