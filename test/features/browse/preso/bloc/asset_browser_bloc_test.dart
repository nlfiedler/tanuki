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
import 'package:tanuki/core/domain/usecases/query_assets.dart';
import 'package:tanuki/core/error/failures.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_browser_bloc.dart';

class MockEntityRepository extends Mock implements EntityRepository {}

void main() {
  late MockEntityRepository mockEntityRepository;
  late QueryAssets usecase;

  const chosenYear = 2009;
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
    const SearchParams dummy = SearchParams();
    registerFallbackValue(dummy);
  });

  group('toggle selector cases', () {
    setUp(() {
      mockEntityRepository = MockEntityRepository();
      usecase = QueryAssets(mockEntityRepository);
      when(() => mockEntityRepository.queryAssets(any(), any(), any()))
          .thenAnswer((_) async => Ok(tQueryResults));
    });

    blocTest(
      'emits [] when nothing is added',
      build: () => AssetBrowserBloc(usecase: usecase),
      expect: () => [],
    );

    blocTest(
      'emits [Loading, Loaded] when LoadInitialAssets is added',
      build: () => AssetBrowserBloc(usecase: usecase),
      act: (AssetBrowserBloc bloc) => bloc.add(LoadInitialAssets()),
      expect: () => [
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: const [],
          locations: const [],
          selectedYear: 0,
          selectedSeason: null,
          lastPage: 1,
          pageSize: 18,
        )
      ],
    );

    blocTest(
      'emits [] when ToggleTag is added w/o LoadInitial',
      build: () => AssetBrowserBloc(usecase: usecase),
      act: (AssetBrowserBloc bloc) =>
          bloc.add(SelectTags(tags: const ['cats'])),
      expect: () => [],
    );

    blocTest(
      'emits [Loading, Loaded, x2] when Initial + ToggleTag is added',
      build: () => AssetBrowserBloc(usecase: usecase),
      act: (AssetBrowserBloc bloc) {
        bloc.add(LoadInitialAssets());
        bloc.add(SelectTags(tags: const ['cats']));
        return;
      },
      expect: () => [
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: const [],
          locations: const [],
          selectedYear: 0,
          selectedSeason: null,
          lastPage: 1,
          pageSize: 18,
        ),
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: const ['cats'],
          locations: const [],
          selectedYear: 0,
          selectedSeason: null,
          lastPage: 1,
          pageSize: 18,
        ),
      ],
    );

    blocTest(
      'emits [] when ToggleLocation is added w/o LoadInitial',
      build: () => AssetBrowserBloc(usecase: usecase),
      act: (AssetBrowserBloc bloc) =>
          bloc.add(SelectLocations(locations: const ['hawaii'])),
      expect: () => [],
    );

    blocTest(
      'emits [Loading, Loaded, x2] when Initial + ToggleLocation is added',
      build: () => AssetBrowserBloc(usecase: usecase),
      act: (AssetBrowserBloc bloc) {
        bloc.add(LoadInitialAssets());
        bloc.add(SelectLocations(locations: const ['hawaii']));
        return;
      },
      expect: () => [
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: const [],
          locations: const [],
          selectedYear: 0,
          selectedSeason: null,
          lastPage: 1,
          pageSize: 18,
        ),
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: const [],
          locations: const ['hawaii'],
          selectedYear: 0,
          selectedSeason: null,
          lastPage: 1,
          pageSize: 18,
        ),
      ],
    );

    blocTest(
      'emits [Loading, Loaded, x2] when Initial + SelectYear is added',
      build: () => AssetBrowserBloc(usecase: usecase),
      act: (AssetBrowserBloc bloc) {
        bloc.add(LoadInitialAssets());
        bloc.add(SelectYear(year: chosenYear));
        return;
      },
      expect: () => [
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: const [],
          locations: const [],
          selectedYear: 0,
          selectedSeason: null,
          lastPage: 1,
          pageSize: 18,
        ),
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: const [],
          locations: const [],
          selectedYear: chosenYear,
          selectedSeason: null,
          lastPage: 1,
          pageSize: 18,
        ),
      ],
    );

    blocTest(
      'emits [Loading, Loaded, x2] when Initial + SelectSeason is added',
      build: () => AssetBrowserBloc(usecase: usecase),
      act: (AssetBrowserBloc bloc) {
        bloc.add(LoadInitialAssets());
        bloc.add(SelectSeason(season: Season.summer));
        return;
      },
      expect: () => [
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: const [],
          locations: const [],
          selectedYear: 0,
          selectedSeason: null,
          lastPage: 1,
          pageSize: 18,
        ),
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: const [],
          locations: const [],
          selectedYear: DateTime.now().year,
          selectedSeason: Season.summer,
          lastPage: 1,
          pageSize: 18,
        ),
      ],
    );

    blocTest(
      'emits [Loading, Loaded, ...] when Initial + Year + Season is added',
      build: () => AssetBrowserBloc(usecase: usecase),
      act: (AssetBrowserBloc bloc) {
        bloc.add(LoadInitialAssets());
        bloc.add(SelectYear(year: chosenYear));
        bloc.add(SelectSeason(season: Season.summer));
        return;
      },
      expect: () => [
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: const [],
          locations: const [],
          selectedYear: 0,
          selectedSeason: null,
          lastPage: 1,
          pageSize: 18,
        ),
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: const [],
          locations: const [],
          selectedYear: chosenYear,
          selectedSeason: null,
          lastPage: 1,
          pageSize: 18,
        ),
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: const [],
          locations: const [],
          selectedYear: chosenYear,
          selectedSeason: Season.summer,
          lastPage: 1,
          pageSize: 18,
        ),
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
      usecase = QueryAssets(mockEntityRepository);
      when(() => mockEntityRepository.queryAssets(any(), any(), any()))
          .thenAnswer((_) async => Ok(manyQueryResults));
    });

    blocTest(
      'emits [Loading, Loaded, x2] when Initial + ShowPage is added',
      build: () => AssetBrowserBloc(usecase: usecase),
      act: (AssetBrowserBloc bloc) {
        bloc.add(LoadInitialAssets());
        bloc.add(ShowPage(page: 10));
        return;
      },
      expect: () => [
        Loading(),
        Loaded(
          results: manyQueryResults,
          pageNumber: 1,
          tags: const [],
          locations: const [],
          selectedYear: 0,
          selectedSeason: null,
          lastPage: 5,
          pageSize: 18,
        ),
        Loading(),
        Loaded(
          results: manyQueryResults,
          pageNumber: 10,
          tags: const [],
          locations: const [],
          selectedYear: 0,
          selectedSeason: null,
          lastPage: 5,
          pageSize: 18,
        ),
      ],
    );

    blocTest(
      'page number resets when Initial + ShowPage + ToggleTag is added',
      build: () => AssetBrowserBloc(usecase: usecase),
      act: (AssetBrowserBloc bloc) {
        bloc.add(LoadInitialAssets());
        bloc.add(ShowPage(page: 10));
        bloc.add(SelectTags(tags: const ['cats']));
        return;
      },
      expect: () => [
        Loading(),
        Loaded(
          results: manyQueryResults,
          pageNumber: 1,
          tags: const [],
          locations: const [],
          selectedYear: 0,
          selectedSeason: null,
          lastPage: 5,
          pageSize: 18,
        ),
        Loading(),
        Loaded(
          results: manyQueryResults,
          pageNumber: 10,
          tags: const [],
          locations: const [],
          selectedYear: 0,
          selectedSeason: null,
          lastPage: 5,
          pageSize: 18,
        ),
        Loading(),
        Loaded(
          results: manyQueryResults,
          pageNumber: 1,
          tags: const ['cats'],
          locations: const [],
          selectedYear: 0,
          selectedSeason: null,
          lastPage: 5,
          pageSize: 18,
        ),
      ],
    );
  });

  group('pagination case: zero', () {
    const zeroQueryResults = QueryResults(results: [], count: 0);

    setUp(() {
      mockEntityRepository = MockEntityRepository();
      usecase = QueryAssets(mockEntityRepository);
      when(() => mockEntityRepository.queryAssets(any(), any(), any()))
          .thenAnswer((_) async => const Ok(zeroQueryResults));
    });

    blocTest(
      'emits [Loading, Loaded] when Initial is added',
      build: () => AssetBrowserBloc(usecase: usecase),
      act: (AssetBrowserBloc bloc) => bloc.add(LoadInitialAssets()),
      expect: () => [
        Loading(),
        Loaded(
          results: zeroQueryResults,
          pageNumber: 0,
          tags: const [],
          locations: const [],
          selectedYear: 0,
          selectedSeason: null,
          lastPage: 0,
          pageSize: 18,
        ),
      ],
    );
  });

  group('error cases', () {
    setUp(() {
      mockEntityRepository = MockEntityRepository();
      usecase = QueryAssets(mockEntityRepository);
      when(() => mockEntityRepository.queryAssets(any(), any(), any()))
          .thenAnswer((_) async => const Err(ServerFailure('oh no!')));
    });

    blocTest(
      'emits [Loading, Error] when LoadInitialAssets is added',
      build: () => AssetBrowserBloc(usecase: usecase),
      act: (AssetBrowserBloc bloc) => bloc.add(LoadInitialAssets()),
      expect: () => [Loading(), Error(message: 'ServerFailure(oh no!)')],
    );
  });
}
