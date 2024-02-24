//
// Copyright (c) 2024 Nathan Fiedler
//
import 'dart:convert';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/data/models/search_model.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('SearchParamsModel', () {
    final tSearchParamsModel = SearchParamsModel(
      tags: const ['clowns', 'snakes'],
      locations: const ['batcave'],
      mediaType: const Some('image/jpeg'),
      after: const None(),
      before: Some(DateTime.utc(2020, 5, 24)),
    );

    const jsonInput = r'''
      {
        "tags": ["clowns", "snakes"],
        "locations": ["batcave"],
        "mediaType": "image/jpeg",
        "after": null,
        "before": "2020-05-24T00:00:00.0Z"
      }
    ''';

    test(
      'should be a subclass of SearchParams entity',
      () async {
        // assert
        expect(tSearchParamsModel, isA<SearchParams>());
      },
    );

    group('fromJson', () {
      test(
        'should return a valid attribute when the JSON is valid',
        () async {
          // arrange
          final Map<String, dynamic> jsonMap = json.decode(jsonInput);
          // act
          final result = SearchParamsModel.fromJson(jsonMap);
          // assert
          expect(result, tSearchParamsModel);
        },
      );
    });

    group('toJson', () {
      test(
        'should return a JSON map containing the proper data',
        () async {
          // act
          final result = tSearchParamsModel.toJson();
          // assert
          final expectedMap = {
            'tags': ['clowns', 'snakes'],
            'locations': ['batcave'],
            'after': null,
            'before': '2020-05-24T00:00:00.000Z',
            'filename': null,
            'mediaType': 'image/jpeg',
            'sortField': null,
            'sortOrder': null,
          };
          expect(result, expectedMap);
        },
      );
    });

    group('toJson and then fromJson', () {
      test('should convert with all non-null options', () {
        // arrange
        final model = SearchParamsModel(
          tags: const ['clowns', 'snakes'],
          locations: const ['batcave'],
          mediaType: const Some('image/jpeg'),
          filename: const Some('catmouse.jpg'),
          after: Some(DateTime.utc(2000, 10, 12)),
          before: Some(DateTime.utc(2020, 5, 24)),
          sortField: const Some(SortField.date),
          sortOrder: const Some(SortOrder.ascending),
        );
        // act
        final result = SearchParamsModel.fromJson(model.toJson());
        // assert
        expect(result, equals(model));
      });

      test('should convert with all null options', () {
        // arrange
        const model = SearchParamsModel();
        // act
        final result = SearchParamsModel.fromJson(model.toJson());
        // assert
        expect(result, equals(model));
      });
    });
  });

  group('SearchResultModel', () {
    const uniqueId = 'MjAyMC8wNS8yNC8x-mini-N5emVhamE4ajZuLmpwZw==';
    final tSearchResultModel = SearchResultModel(
      id: uniqueId,
      filename: 'catmouse_1280p.jpg',
      mediaType: 'image/jpeg',
      location: Some(AssetLocation.from('outdoors')),
      datetime: DateTime.utc(2020, 5, 24, 18, 02, 15),
    );

    const jsonInput = r'''
      {
        "id": "MjAyMC8wNS8yNC8x-mini-N5emVhamE4ajZuLmpwZw==",
        "filename": "catmouse_1280p.jpg",
        "mediaType": "image/jpeg",
        "location": {"label": "outdoors", "city": null, "region": null},
        "datetime": "2020-05-24T18:02:15.829336+00:00"
      }
    ''';

    test(
      'should be a subclass of SearchResult entity',
      () async {
        // assert
        expect(tSearchResultModel, isA<SearchResult>());
      },
    );

    group('fromJson', () {
      test(
        'should return a valid attribute when the JSON is valid',
        () async {
          // arrange
          final Map<String, dynamic> jsonMap = json.decode(jsonInput);
          // act
          final result = SearchResultModel.fromJson(jsonMap);
          // assert
          expect(result, tSearchResultModel);
        },
      );
    });

    group('toJson', () {
      test(
        'should return a JSON map containing the proper data',
        () async {
          // act
          final result = tSearchResultModel.toJson();
          // assert
          final expectedMap = {
            'id': uniqueId,
            'filename': 'catmouse_1280p.jpg',
            'mediaType': 'image/jpeg',
            'location': {'label': 'outdoors', 'city': null, 'region': null},
            'datetime': '2020-05-24T18:02:15.000Z',
          };
          expect(result, expectedMap);
        },
      );
    });

    group('toJson and then fromJson', () {
      test('should convert all non-null options', () {
        // arrange
        final model = SearchResultModel(
          id: 'MjAyMC8wNS8yNC8xODAwLzAxZTkzeGp6d25keajZuLmpwZw==',
          filename: 'mousecat_1280p.jpg',
          mediaType: 'image/jpeg',
          location: const None(),
          datetime: DateTime.utc(2010, 10, 12, 9, 20, 51),
        );
        // act
        final result = SearchResultModel.fromJson(model.toJson());
        // assert
        expect(result, equals(model));
      });
    });
  });

  group('QueryResultsModel', () {
    const uniqueId = 'MjAyMC8wNS8yNC8x-mini-N5emVhamE4ajZuLmpwZw==';
    final tQueryResultsModel = QueryResultsModel(
      results: [
        SearchResultModel(
          id: uniqueId,
          filename: 'catmouse_1280p.jpg',
          mediaType: 'image/jpeg',
          location: Some(AssetLocation.from('outdoors')),
          datetime: DateTime.utc(2020, 5, 24, 18, 02, 15),
        )
      ],
      count: 101,
    );

    const jsonInput = r'''
      {
        "results": [
          {
            "id": "MjAyMC8wNS8yNC8x-mini-N5emVhamE4ajZuLmpwZw==",
            "filename": "catmouse_1280p.jpg",
            "mediaType": "image/jpeg",
            "location": {"label": "outdoors", "city": null, "region": null},
            "datetime": "2020-05-24T18:02:15.829336+00:00"
          }
        ],
        "count": 101
      }
    ''';

    test(
      'should be a subclass of QueryResults entity',
      () async {
        // assert
        expect(tQueryResultsModel, isA<QueryResults>());
      },
    );

    group('fromJson', () {
      test(
        'should return a valid attribute when the JSON is valid',
        () async {
          // arrange
          final Map<String, dynamic> jsonMap = json.decode(jsonInput);
          // act
          final result = QueryResultsModel.fromJson(jsonMap);
          // assert
          expect(result, tQueryResultsModel);
        },
      );
    });

    group('toJson', () {
      test(
        'should return a JSON map containing the proper data',
        () async {
          // act
          final result = tQueryResultsModel.toJson();
          // assert
          final Map<String, dynamic> expectedMap = {
            'results': [
              {
                'id': uniqueId,
                'filename': 'catmouse_1280p.jpg',
                'mediaType': 'image/jpeg',
                'location': {'label': 'outdoors', 'city': null, 'region': null},
                'datetime': '2020-05-24T18:02:15.000Z',
              }
            ],
            'count': 101,
          };
          expect(result, expectedMap);
        },
      );
    });

    group('toJson and then fromJson', () {
      test('should convert all non-null options', () {
        // arrange
        final model = QueryResultsModel(
          results: [
            SearchResultModel(
              id: 'MjAyMC8wNS8yNC8xODAwLzAxZTkzeGp6d25keajZuLmpwZw==',
              filename: 'mousecat_1280p.jpg',
              mediaType: 'image/jpeg',
              location: const None(),
              datetime: DateTime.utc(2010, 10, 12, 9, 20, 51),
            )
          ],
          count: 101,
        );
        // act
        final result = QueryResultsModel.fromJson(model.toJson());
        // assert
        expect(result, equals(model));
      });
    });
  });
}
