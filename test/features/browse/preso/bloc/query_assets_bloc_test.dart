//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/query_assets.dart';
import 'package:tanuki/core/error/failures.dart';
import 'package:tanuki/features/browse/preso/bloc/query_assets_bloc.dart';

class MockEntityRepository extends Mock implements EntityRepository {}

void main() {
  MockEntityRepository mockEntityRepository;
  QueryAssets usecase;

  final tSearchParams = SearchParams(tags: ["mouse"]);
  final tQueryResults = QueryResults(
    results: [
      SearchResult(
        id: 'MjAyMC8wNS8yNC8x-mini-N5emVhamE4ajZuLmpwZw==',
        filename: 'catmouse_1280p.jpg',
        mimetype: 'image/jpeg',
        location: Some('outdoors'),
        datetime: DateTime.utc(2020, 5, 24, 18, 02, 15),
      )
    ],
    count: 1,
  );

  group('normal cases', () {
    setUp(() {
      mockEntityRepository = MockEntityRepository();
      usecase = QueryAssets(mockEntityRepository);
      when(mockEntityRepository.queryAssets(any, any, any))
          .thenAnswer((_) async => Ok(tQueryResults));
    });

    blocTest(
      'emits [] when nothing is added',
      build: () => QueryAssetsBloc(usecase: usecase),
      expect: [],
    );

    blocTest(
      'emits [Loading, Loaded] when LoadQueryAssets is added',
      build: () => QueryAssetsBloc(usecase: usecase),
      act: (bloc) => bloc.add(LoadQueryAssets(
        params: tSearchParams,
        count: 10,
        offset: 0,
      )),
      expect: [Loading(), Loaded(results: tQueryResults)],
    );
  });

  group('error cases', () {
    setUp(() {
      mockEntityRepository = MockEntityRepository();
      usecase = QueryAssets(mockEntityRepository);
      when(mockEntityRepository.queryAssets(any, any, any))
          .thenAnswer((_) async => Err(ServerFailure('oh no!')));
    });

    blocTest(
      'emits [Loading, Error] when LoadQueryAssets is added',
      build: () => QueryAssetsBloc(usecase: usecase),
      act: (bloc) => bloc.add(LoadQueryAssets(
        params: tSearchParams,
        count: 10,
        offset: 0,
      )),
      expect: [Loading(), Error(message: 'ServerFailure(oh no!)')],
    );
  });
}
