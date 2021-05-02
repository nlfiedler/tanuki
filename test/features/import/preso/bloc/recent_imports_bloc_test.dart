//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:mockito/annotations.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/query_recents.dart';
import 'package:tanuki/core/error/failures.dart';
import 'package:tanuki/features/import/preso/bloc/recent_imports_bloc.dart';
import './recent_imports_bloc_test.mocks.dart';

@GenerateMocks([EntityRepository])
void main() {
  late MockEntityRepository mockAssetRepository;
  late QueryRecents usecase;

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
      mockAssetRepository = MockEntityRepository();
      usecase = QueryRecents(mockAssetRepository);
      when(mockAssetRepository.queryRecents(any))
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
        )
      ],
    );
  });

  group('error cases', () {
    setUp(() {
      mockAssetRepository = MockEntityRepository();
      usecase = QueryRecents(mockAssetRepository);
      when(mockAssetRepository.queryRecents(any))
          .thenAnswer((_) async => Err(ServerFailure('oh no!')));
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
