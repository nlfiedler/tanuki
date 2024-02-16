//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/domain/entities/input.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/bulk_update.dart';
import 'package:tanuki/core/error/failures.dart';

class MockEntityRepository extends Mock implements EntityRepository {}

void main() {
  late BulkUpdate usecase;
  late MockEntityRepository mockEntityRepository;

  setUp(() {
    mockEntityRepository = MockEntityRepository();
    usecase = BulkUpdate(mockEntityRepository);
  });

  setUpAll(() {
    // mocktail needs a fallback for any() that involves custom types
    const List<AssetInputId> dummy = [];
    registerFallbackValue(dummy);
  });

  test(
    'should update assets in the repository',
    () async {
      // arrange
      const Result<int, Failure> expected = Ok(32);
      when(() => mockEntityRepository.bulkUpdate(any()))
          .thenAnswer((_) async => const Ok(32));
      // act
      final inputId = AssetInputId(
        id: 'asset123',
        input: AssetInput(
          tags: const ['clowns', 'snakes'],
          caption: const Some('#snakes and #clowns are in my @batcave'),
          location: Some(AssetLocation.from('batcave')),
          datetime: Some(DateTime.utc(2003, 8, 30)),
          mimetype: const Some('image/jpeg'),
          filename: const Some('img_1234.jpg'),
        ),
      );
      final params = Params(assets: [inputId]);
      final result = await usecase(params);
      // assert
      expect(result, expected);
      verify(() => mockEntityRepository.bulkUpdate(params.assets));
      verifyNoMoreInteractions(mockEntityRepository);
    },
  );
}
