//
// Copyright (c) 2024 Nathan Fiedler
//
import 'dart:convert';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/data/models/asset_model.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('AssetModel', () {
    final tAssetModel = AssetModel(
      id: 'asset123',
      checksum: 'sha1-cafebabe',
      filename: 'img_1234.jpg',
      filesize: 1048576,
      datetime: DateTime.utc(2003, 8, 30),
      mediaType: 'image/jpeg',
      tags: const ['clowns', 'snakes'],
      userdate: const None(),
      caption: const Some('#snakes and #clowns are in my @batcave'),
      location: const Some(AssetLocationModel(
        label: Some('batcave'),
        city: None(),
        region: None(),
      )),
    );

    test(
      'should be a subclass of Asset entity',
      () async {
        // assert
        expect(tAssetModel, isA<Asset>());
      },
    );

    group('fromJson', () {
      test(
        'should return a valid attribute when the JSON is valid',
        () async {
          // arrange
          final Map<String, dynamic> jsonMap = json.decode(r'''
            {
              "id": "asset123",
              "checksum": "sha1-cafebabe",
              "filename": "img_1234.jpg",
              "filesize": "1048576",
              "datetime": "2003-08-30T00:00:00.0Z",
              "mediaType": "image/jpeg",
              "tags": ["clowns", "snakes"],
              "userdate": null,
              "caption": "#snakes and #clowns are in my @batcave",
              "location": { "label": "batcave", "city": null, "region": null }
            }
          ''');
          // act
          final result = AssetModel.fromJson(jsonMap);
          // assert
          expect(result, tAssetModel);
        },
      );
    });

    group('toJson', () {
      test(
        'should return a JSON map containing the proper data',
        () async {
          // act
          final result = tAssetModel.toJson();
          // assert
          final expectedMap = {
            'id': 'asset123',
            'checksum': 'sha1-cafebabe',
            'filename': 'img_1234.jpg',
            'filesize': '1048576',
            'datetime': '2003-08-30T00:00:00.000Z',
            'mediaType': 'image/jpeg',
            'tags': ['clowns', 'snakes'],
            'userdate': null,
            'caption': '#snakes and #clowns are in my @batcave',
            'location': {'label': "batcave", 'city': null, 'region': null},
          };
          expect(result, expectedMap);
        },
      );
    });
  });
}
