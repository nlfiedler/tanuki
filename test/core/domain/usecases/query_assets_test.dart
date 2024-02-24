//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/query_assets.dart';
import 'package:tanuki/core/error/failures.dart';

class MockEntityRepository extends Mock implements EntityRepository {}

void main() {
  late QueryAssets usecase;
  late MockEntityRepository mockEntityRepository;

  setUp(() {
    mockEntityRepository = MockEntityRepository();
    usecase = QueryAssets(mockEntityRepository);
  });

  setUpAll(() {
    // mocktail needs a fallback for any() that involves custom types
    const SearchParams dummy = SearchParams();
    registerFallbackValue(dummy);
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
            mediaType: 'image/jpeg',
            location: Some(AssetLocation.from('outdoors')),
            datetime: DateTime.utc(2020, 5, 24, 18, 02, 15),
          )
        ],
        count: 1,
      );
      final Result<QueryResults, Failure> expected = Ok(expectedResults);
      when(() => mockEntityRepository.queryAssets(any(), any(), any()))
          .thenAnswer((_) async => Ok(expectedResults));
      // act
      const searchParams = SearchParams(tags: ['mouse']);
      const params = Params(
        params: searchParams,
        count: 10,
        offset: 0,
      );
      final result = await usecase(params);
      // assert
      expect(result, expected);
      expect(result.unwrap().results, equals(expectedResults.results));
      verify(() => mockEntityRepository.queryAssets(searchParams, 10, 0));
      verifyNoMoreInteractions(mockEntityRepository);
    },
  );
}
