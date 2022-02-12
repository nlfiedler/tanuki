//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:mockito/annotations.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/query_assets.dart';
import 'package:tanuki/core/error/failures.dart';
import './query_assets_test.mocks.dart';

@GenerateMocks([EntityRepository])
void main() {
  late QueryAssets usecase;
  late MockEntityRepository mockEntityRepository;

  setUp(() {
    mockEntityRepository = MockEntityRepository();
    usecase = QueryAssets(mockEntityRepository);
  });

  test(
    'should query assets from the repository',
    () async {
      // arrange
      final expectedResults = QueryResults(
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
      final Result<QueryResults, Failure> expected = Ok(expectedResults);
      when(mockEntityRepository.queryAssets(any, any, any))
          .thenAnswer((_) async => Ok(expectedResults));
      // act
      final searchParams = SearchParams(tags: ['mouse']);
      final params = Params(
        params: searchParams,
        count: 10,
        offset: 0,
      );
      final result = await usecase(params);
      // assert
      expect(result, expected);
      expect(result.unwrap().results, equals(expectedResults.results));
      verify(mockEntityRepository.queryAssets(searchParams, 10, 0));
      verifyNoMoreInteractions(mockEntityRepository);
    },
  );
}
