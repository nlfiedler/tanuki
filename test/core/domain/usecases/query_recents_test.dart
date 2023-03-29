//
// Copyright (c) 2023 Nathan Fiedler
//
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/query_recents.dart';
import 'package:tanuki/core/error/failures.dart';

class MockEntityRepository extends Mock implements EntityRepository {}

void main() {
  late QueryRecents usecase;
  late MockEntityRepository mockEntityRepository;

  setUp(() {
    mockEntityRepository = MockEntityRepository();
    usecase = QueryRecents(mockEntityRepository);
  });

  setUpAll(() {
    // mocktail needs a fallback for any() that involves custom types
    const Option<DateTime> dummy = None();
    registerFallbackValue(dummy);
    const Option<int> dummyInt = None();
    registerFallbackValue(dummyInt);
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
      when(() => mockEntityRepository.queryRecents(any(), any(), any()))
          .thenAnswer((_) async => Ok(expectedResults));
      // act
      final Option<DateTime> since = Some(DateTime.now());
      final params =
          Params(since: since, count: const None(), offset: const None());
      final result = await usecase(params);
      // assert
      expect(result, expected);
      expect(result.unwrap().results, equals(expectedResults.results));
      verify(() =>
          mockEntityRepository.queryRecents(since, const None(), const None()));
      verifyNoMoreInteractions(mockEntityRepository);
    },
  );
}
