//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:mockito/annotations.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/input.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/bulk_update.dart';
import 'package:tanuki/core/error/failures.dart';
import './bulk_update_test.mocks.dart';

@GenerateMocks([EntityRepository])
void main() {
  late BulkUpdate usecase;
  late MockEntityRepository mockEntityRepository;

  setUp(() {
    mockEntityRepository = MockEntityRepository();
    usecase = BulkUpdate(mockEntityRepository);
  });

  test(
    'should update assets in the repository',
    () async {
      // arrange
      final Result<int, Failure> expected = Ok(32);
      when(mockEntityRepository.bulkUpdate(any))
          .thenAnswer((_) async => Ok(32));
      // act
      final inputId = AssetInputId(
        id: 'asset123',
        input: AssetInput(
          tags: ['clowns', 'snakes'],
          caption: Some('#snakes and #clowns are in my @batcave'),
          location: Some('batcave'),
          datetime: Some(DateTime.utc(2003, 8, 30)),
          mimetype: Some('image/jpeg'),
          filename: Some('img_1234.jpg'),
        ),
      );
      final params = Params(assets: [inputId]);
      final result = await usecase(params);
      // assert
      expect(result, expected);
      verify(mockEntityRepository.bulkUpdate(params.assets));
      verifyNoMoreInteractions(mockEntityRepository);
    },
  );
}
