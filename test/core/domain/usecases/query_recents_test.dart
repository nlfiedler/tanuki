//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/query_recents.dart';

class MockEntityRepository extends Mock implements EntityRepository {}

void main() {
  QueryRecents usecase;
  MockEntityRepository mockEntityRepository;

  setUp(() {
    mockEntityRepository = MockEntityRepository();
    usecase = QueryRecents(mockEntityRepository);
  });

  test(
    'should query assets from the repository',
    () async {
      // arrange
      final expected = QueryResults(
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
      when(mockEntityRepository.queryRecents(any))
          .thenAnswer((_) async => Ok(expected));
      // act
      final Option<DateTime> since = Some(DateTime.now());
      final params = Params(since: since);
      final result = await usecase(params);
      // assert
      expect(result, Ok(expected));
      expect(result.unwrap().results, equals(expected.results));
      verify(mockEntityRepository.queryRecents(since));
      verifyNoMoreInteractions(mockEntityRepository);
    },
  );
}
