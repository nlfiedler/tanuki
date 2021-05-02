//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:mockito/annotations.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/get_asset.dart';
import './get_asset_test.mocks.dart';

@GenerateMocks([EntityRepository])
void main() {
  late GetAsset usecase;
  late MockEntityRepository mockEntityRepository;

  setUp(() {
    mockEntityRepository = MockEntityRepository();
    usecase = GetAsset(mockEntityRepository);
  });

  test(
    'should get asset from the repository',
    () async {
      // arrange
      final expected = Asset(
        id: 'MjAyMC8wNS8yNC8x-mini-N5emVhamE4ajZuLmpwZw==',
        checksum: 'sha256-34641209e88f3a59b-mini-2dfdcb00f8a533ac80ba',
        filename: 'catmouse_1280p.jpg',
        filesize: 160852,
        datetime: DateTime.utc(2020, 5, 24, 18, 02, 15),
        mimetype: 'image/jpeg',
        tags: ['cat', 'mouse'],
        userdate: None(),
        caption: Some('#cat @outdoors #mouse'),
        location: Some('outdoors'),
      );
      when(mockEntityRepository.getAsset(any))
          .thenAnswer((_) async => Ok(expected));
      // act
      final params = Params(
        assetId: 'MjAyMC8wNS8yNC8x-mini-N5emVhamE4ajZuLmpwZw==',
      );
      final result = await usecase(params);
      // assert
      expect(result, Ok(expected));
      expect(result.unwrap(), equals(expected));
      verify(mockEntityRepository.getAsset(params.assetId));
      verifyNoMoreInteractions(mockEntityRepository);
    },
  );
}
