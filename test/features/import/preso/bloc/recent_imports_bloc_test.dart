//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/query_recents.dart';
import 'package:tanuki/core/error/failures.dart';
import 'package:tanuki/features/import/preso/bloc/recent_imports_bloc.dart';

class MockEntityRepository extends Mock implements EntityRepository {}

void main() {
  late MockEntityRepository mockEntityRepository;
  late QueryRecents usecase;

  final tQueryResults = QueryResults(
    results: [
      SearchResult(
        id: 'MjAyMC8wNS8yNC8x-mini-N5emVhamE4ajZuLmpwZw==',
        filename: 'catmouse_1280p.jpg',
        mediaType: 'image/jpeg',
        location: Some(AssetLocation.from('outdoors')),
        datetime: DateTime.utc(2020, 5, 24, 18, 02, 15),
      )
    ],
    count: 1,
  );

  setUpAll(() {
    // mocktail needs a fallback for any() that involves custom types
    const SearchParams dummySearchParams = SearchParams();
    registerFallbackValue(dummySearchParams);
    const Option<DateTime> dummy = None();
    registerFallbackValue(dummy);
    const Option<int> dummyInt = None();
    registerFallbackValue(dummyInt);
  });

  group('normal cases', () {
    setUp(() {
      mockEntityRepository = MockEntityRepository();
      usecase = QueryRecents(mockEntityRepository);
      when(() => mockEntityRepository.queryRecents(any(), any(), any()))
          .thenAnswer((_) async => Ok(tQueryResults));
    });

    blocTest(
      'emits [] when nothing is added',
      build: () => RecentImportsBloc(usecase: usecase),
      expect: () => [],
    );

    blocTest(
      'emits [Loading, Loaded] when FindRecents is added',
      build: () => RecentImportsBloc(usecase: usecase),
      act: (RecentImportsBloc bloc) =>
          bloc.add(FindRecents(range: RecentTimeRange.day)),
      expect: () => [
        Loading(),
        Loaded(
          results: tQueryResults,
          range: RecentTimeRange.day,
          pageSize: 18,
          pageNumber: 1,
          lastPage: 1,
        )
      ],
    );

    blocTest(
      'emits [Loading, Loaded] when RefreshResults is added',
      build: () => RecentImportsBloc(usecase: usecase),
      act: (RecentImportsBloc bloc) => bloc.add(RefreshResults()),
      expect: () => [
        Loading(),
        Loaded(
          results: tQueryResults,
          range: RecentTimeRange.day,
          pageSize: 18,
          pageNumber: 1,
          lastPage: 1,
        )
      ],
    );
  });

  group('pagination case: many', () {
    final manyQueryResults = QueryResults(
      results: [
        SearchResult(
          id: 'MjAyMC8wNS8yNC8x-mini-N5emVhamE4ajZuLmpwZw==',
          filename: 'catmouse_1280p.jpg',
          mediaType: 'image/jpeg',
          location: Some(AssetLocation.from('outdoors')),
          datetime: DateTime.utc(2020, 5, 24, 18, 02, 15),
        )
      ],
      count: 85,
    );

    setUp(() {
      mockEntityRepository = MockEntityRepository();
      usecase = QueryRecents(mockEntityRepository);
      when(() => mockEntityRepository.queryRecents(any(), any(), any()))
          .thenAnswer((_) async => Ok(manyQueryResults));
    });

    blocTest(
      'emits [Loading, Loaded, x2] when Initial + ShowPage is added',
      build: () => RecentImportsBloc(usecase: usecase),
      act: (RecentImportsBloc bloc) {
        bloc.add(FindRecents(range: RecentTimeRange.day));
        bloc.add(ShowPage(page: 10));
        return;
      },
      expect: () => [
        Loading(),
        Loaded(
          results: manyQueryResults,
          range: RecentTimeRange.day,
          pageSize: 18,
          pageNumber: 1,
          lastPage: 5,
        ),
        Loading(),
        Loaded(
          results: manyQueryResults,
          range: RecentTimeRange.day,
          pageSize: 18,
          pageNumber: 10,
          lastPage: 5,
        ),
      ],
    );

    blocTest(
      'page number resets when refreshing results',
      build: () => RecentImportsBloc(usecase: usecase),
      act: (RecentImportsBloc bloc) {
        bloc.add(FindRecents(range: RecentTimeRange.day));
        bloc.add(ShowPage(page: 10));
        bloc.add(RefreshResults());
        return;
      },
      expect: () => [
        Loading(),
        Loaded(
          results: manyQueryResults,
          range: RecentTimeRange.day,
          pageSize: 18,
          pageNumber: 1,
          lastPage: 5,
        ),
        Loading(),
        Loaded(
          results: manyQueryResults,
          range: RecentTimeRange.day,
          pageSize: 18,
          pageNumber: 10,
          lastPage: 5,
        ),
        Loading(),
        Loaded(
          results: manyQueryResults,
          range: RecentTimeRange.day,
          pageSize: 18,
          pageNumber: 1,
          lastPage: 5,
        ),
      ],
    );

    blocTest(
      'page number resets when changing time range',
      build: () => RecentImportsBloc(usecase: usecase),
      act: (RecentImportsBloc bloc) {
        bloc.add(FindRecents(range: RecentTimeRange.day));
        bloc.add(ShowPage(page: 10));
        bloc.add(FindRecents(range: RecentTimeRange.week));
        return;
      },
      expect: () => [
        Loading(),
        Loaded(
          results: manyQueryResults,
          range: RecentTimeRange.day,
          pageSize: 18,
          pageNumber: 1,
          lastPage: 5,
        ),
        Loading(),
        Loaded(
          results: manyQueryResults,
          range: RecentTimeRange.day,
          pageSize: 18,
          pageNumber: 10,
          lastPage: 5,
        ),
        Loading(),
        Loaded(
          results: manyQueryResults,
          range: RecentTimeRange.week,
          pageSize: 18,
          pageNumber: 1,
          lastPage: 5,
        ),
      ],
    );
  });

  group('pagination case: zero', () {
    const zeroQueryResults = QueryResults(results: [], count: 0);

    setUp(() {
      mockEntityRepository = MockEntityRepository();
      usecase = QueryRecents(mockEntityRepository);
      when(() => mockEntityRepository.queryRecents(any(), any(), any()))
          .thenAnswer((_) async => const Ok(zeroQueryResults));
    });

    blocTest(
      'emits [Loading, Loaded] when Initial is added',
      build: () => RecentImportsBloc(usecase: usecase),
      act: (RecentImportsBloc bloc) =>
          bloc.add(FindRecents(range: RecentTimeRange.day)),
      expect: () => [
        Loading(),
        Loaded(
          results: zeroQueryResults,
          range: RecentTimeRange.day,
          pageSize: 18,
          pageNumber: 0,
          lastPage: 0,
        ),
      ],
    );
  });

  group('error cases', () {
    setUp(() {
      mockEntityRepository = MockEntityRepository();
      usecase = QueryRecents(mockEntityRepository);
      when(() => mockEntityRepository.queryRecents(any(), any(), any()))
          .thenAnswer((_) async => const Err(ServerFailure('oh no!')));
    });

    blocTest(
      'emits [Loading, Error] when FindRecents is added',
      build: () => RecentImportsBloc(usecase: usecase),
      act: (RecentImportsBloc bloc) =>
          bloc.add(FindRecents(range: RecentTimeRange.day)),
      expect: () => [Loading(), Error(message: 'ServerFailure(oh no!)')],
    );
  });
}
