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
import 'package:tanuki/features/browse/preso/bloc/asset_browser_bloc.dart';

class MockEntityRepository extends Mock implements EntityRepository {}

void main() {
  MockEntityRepository mockEntityRepository;
  QueryAssets usecase;

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

  group('toggle selector cases', () {
    setUp(() {
      mockEntityRepository = MockEntityRepository();
      usecase = QueryAssets(mockEntityRepository);
      when(mockEntityRepository.queryAssets(any, any, any))
          .thenAnswer((_) async => Ok(tQueryResults));
    });

    blocTest(
      'emits [] when nothing is added',
      build: () => AssetBrowserBloc(usecase: usecase),
      expect: [],
    );

    blocTest(
      'emits [Loading, Loaded] when LoadInitialAssets is added',
      build: () => AssetBrowserBloc(usecase: usecase),
      act: (bloc) => bloc.add(LoadInitialAssets()),
      expect: [
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: [],
          locations: [],
          selectedYear: None(),
        )
      ],
    );

    blocTest(
      'emits [] when ToggleTag is added w/o LoadInitial',
      build: () => AssetBrowserBloc(usecase: usecase),
      act: (bloc) => bloc.add(ToggleTag(tag: 'cats')),
      expect: [],
    );

    blocTest(
      'emits [Loading, Loaded, x2] when Initial + ToggleTag is added',
      build: () => AssetBrowserBloc(usecase: usecase),
      act: (bloc) {
        bloc.add(LoadInitialAssets());
        bloc.add(ToggleTag(tag: 'cats'));
        return;
      },
      expect: [
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: [],
          locations: [],
          selectedYear: None(),
        ),
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: ['cats'],
          locations: [],
          selectedYear: None(),
        ),
      ],
    );

    blocTest(
      'emits [] when ToggleLocation is added w/o LoadInitial',
      build: () => AssetBrowserBloc(usecase: usecase),
      act: (bloc) => bloc.add(ToggleLocation(location: 'hawaii')),
      expect: [],
    );

    blocTest(
      'emits [Loading, Loaded, x2] when Initial + ToggleLocation is added',
      build: () => AssetBrowserBloc(usecase: usecase),
      act: (bloc) {
        bloc.add(LoadInitialAssets());
        bloc.add(ToggleLocation(location: 'hawaii'));
        return;
      },
      expect: [
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: [],
          locations: [],
          selectedYear: None(),
        ),
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: [],
          locations: ['hawaii'],
          selectedYear: None(),
        ),
      ],
    );

    blocTest(
      'emits [Loading, Loaded, x2] when Initial + ToggleYear is added',
      build: () => AssetBrowserBloc(usecase: usecase),
      act: (bloc) {
        bloc.add(LoadInitialAssets());
        bloc.add(ToggleYear(year: 2009));
        return;
      },
      expect: [
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: [],
          locations: [],
          selectedYear: None(),
        ),
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: [],
          locations: [],
          selectedYear: Some(2009),
        ),
      ],
    );

    blocTest(
      'unselects year if toggled twice',
      build: () => AssetBrowserBloc(usecase: usecase),
      act: (bloc) {
        bloc.add(LoadInitialAssets());
        bloc.add(ToggleYear(year: 2009));
        bloc.add(ToggleYear(year: 2009));
        return;
      },
      expect: [
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: [],
          locations: [],
          selectedYear: None(),
        ),
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: [],
          locations: [],
          selectedYear: Some(2009),
        ),
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: [],
          locations: [],
          selectedYear: None(),
        ),
      ],
    );
  });

  group('pagination cases', () {
    setUp(() {
      mockEntityRepository = MockEntityRepository();
      usecase = QueryAssets(mockEntityRepository);
      when(mockEntityRepository.queryAssets(any, any, any))
          .thenAnswer((_) async => Ok(tQueryResults));
    });

    blocTest(
      'emits [Loading, Loaded, x2] when Initial + ShowPage is added',
      build: () => AssetBrowserBloc(usecase: usecase),
      act: (bloc) {
        bloc.add(LoadInitialAssets());
        bloc.add(ShowPage(page: 10));
        return;
      },
      expect: [
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: [],
          locations: [],
          selectedYear: None(),
        ),
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 10,
          tags: [],
          locations: [],
          selectedYear: None(),
        ),
      ],
    );

    blocTest(
      'page number resets when Initial + ShowPage + ToggleTag is added',
      build: () => AssetBrowserBloc(usecase: usecase),
      act: (bloc) {
        bloc.add(LoadInitialAssets());
        bloc.add(ShowPage(page: 10));
        bloc.add(ToggleTag(tag: 'cats'));
        return;
      },
      expect: [
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: [],
          locations: [],
          selectedYear: None(),
        ),
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 10,
          tags: [],
          locations: [],
          selectedYear: None(),
        ),
        Loading(),
        Loaded(
          results: tQueryResults,
          pageNumber: 1,
          tags: ['cats'],
          locations: [],
          selectedYear: None(),
        ),
      ],
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
      'emits [Loading, Error] when LoadInitialAssets is added',
      build: () => AssetBrowserBloc(usecase: usecase),
      act: (bloc) => bloc.add(LoadInitialAssets()),
      expect: [Loading(), Error(message: 'ServerFailure(oh no!)')],
    );
  });
}
