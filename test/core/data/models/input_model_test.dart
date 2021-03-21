//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:convert';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/data/models/input_model.dart';
import 'package:tanuki/core/domain/entities/input.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('AssetInputModel', () {
    final tAssetModel = AssetInputModel(
      tags: ['clowns', 'snakes'],
      caption: Some('#snakes and #clowns are in my @batcave'),
      location: Some('batcave'),
      datetime: Some(DateTime.utc(2003, 8, 30)),
      mimetype: Some('image/jpeg'),
      filename: Some('img_1234.jpg'),
    );

    test(
      'should be a subclass of AssetInput entity',
      () async {
        // assert
        expect(tAssetModel, isA<AssetInput>());
      },
    );

    group('fromJson', () {
      test(
        'should return a valid model when the JSON is valid',
        () async {
          // arrange
          final Map<String, dynamic> jsonMap = json.decode(r'''
            {
              "filename": "img_1234.jpg",
              "datetime": "2003-08-30T00:00:00.0Z",
              "mimetype": "image/jpeg",
              "tags": ["clowns", "snakes"],
              "caption": "#snakes and #clowns are in my @batcave",
              "location": "batcave"
            }
          ''');
          // act
          final result = AssetInputModel.fromJson(jsonMap);
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
            'filename': 'img_1234.jpg',
            'datetime': '2003-08-30T00:00:00.000Z',
            'mimetype': 'image/jpeg',
            'tags': ['clowns', 'snakes'],
            'caption': '#snakes and #clowns are in my @batcave',
            'location': 'batcave',
          };
          expect(result, expectedMap);
        },
      );
    });
  });

  group('AssetInputIdModel', () {
    final tAssetModel = AssetInputIdModel(
      id: 'asset123',
      input: AssetInputModel(
        tags: ['clowns', 'snakes'],
        caption: Some('#snakes and #clowns are in my @batcave'),
        location: Some('batcave'),
        datetime: Some(DateTime.utc(2003, 8, 30)),
        mimetype: Some('image/jpeg'),
        filename: Some('img_1234.jpg'),
      ),
    );

    test(
      'should be a subclass of AssetInput entity',
      () async {
        // assert
        expect(tAssetModel, isA<AssetInputId>());
      },
    );

    group('fromJson', () {
      test(
        'should return a valid model when the JSON is valid',
        () async {
          // arrange
          final Map<String, dynamic> jsonMap = json.decode(r'''
            {
              "id": "asset123",
              "input": {
                "filename": "img_1234.jpg",
                "datetime": "2003-08-30T00:00:00.0Z",
                "mimetype": "image/jpeg",
                "tags": ["clowns", "snakes"],
                "caption": "#snakes and #clowns are in my @batcave",
                "location": "batcave"
              }
            }
          ''');
          // act
          final result = AssetInputIdModel.fromJson(jsonMap);
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
            'input': {
              'filename': 'img_1234.jpg',
              'datetime': '2003-08-30T00:00:00.000Z',
              'mimetype': 'image/jpeg',
              'tags': ['clowns', 'snakes'],
              'caption': '#snakes and #clowns are in my @batcave',
              'location': 'batcave',
            }
          };
          expect(result, expectedMap);
        },
      );
    });
  });
}
