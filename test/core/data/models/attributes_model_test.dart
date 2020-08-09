//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:convert';
import 'package:tanuki/core/data/models/attributes_model.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('LocationModel', () {
    final tLocationModel = LocationModel(
      label: 'tokyo',
      count: 806,
    );

    test(
      'should be a subclass of Location entity',
      () async {
        // assert
        expect(tLocationModel, isA<Location>());
      },
    );

    group('fromJson', () {
      test(
        'should return a valid attribute when the JSON is valid',
        () async {
          // arrange
          final Map<String, dynamic> jsonMap =
              json.decode('{ "label": "tokyo", "count": 806 }');
          // act
          final result = LocationModel.fromJson(jsonMap);
          // assert
          expect(result, tLocationModel);
        },
      );
    });

    group('toJson', () {
      test(
        'should return a JSON map containing the proper data',
        () async {
          // act
          final result = tLocationModel.toJson();
          // assert
          final expectedMap = {'label': 'tokyo', 'count': 806};
          expect(result, expectedMap);
        },
      );
    });

    group('toJson and then fromJson', () {
      test('should convert all non-null options', () {
        // arrange
        final model = LocationModel(label: 'london', count: 138);
        // act
        final result = LocationModel.fromJson(model.toJson());
        // assert
        expect(result, equals(model));
      });
    });
  });

  group('TagModel', () {
    final tTagModel = TagModel(
      label: 'kittens',
      count: 806,
    );

    test(
      'should be a subclass of Tag entity',
      () async {
        // assert
        expect(tTagModel, isA<Tag>());
      },
    );

    group('fromJson', () {
      test(
        'should return a valid attribute when the JSON is valid',
        () async {
          // arrange
          final Map<String, dynamic> jsonMap =
              json.decode('{ "label": "kittens", "count": 806 }');
          // act
          final result = TagModel.fromJson(jsonMap);
          // assert
          expect(result, tTagModel);
        },
      );
    });

    group('toJson', () {
      test(
        'should return a JSON map containing the proper data',
        () async {
          // act
          final result = tTagModel.toJson();
          // assert
          final expectedMap = {'label': 'kittens', 'count': 806};
          expect(result, expectedMap);
        },
      );
    });

    group('toJson and then fromJson', () {
      test('should convert all non-null options', () {
        // arrange
        final model = TagModel(label: 'snakes', count: 138);
        // act
        final result = TagModel.fromJson(model.toJson());
        // assert
        expect(result, equals(model));
      });
    });
  });

  group('YearModel', () {
    final tYearModel = YearModel(
      label: '2019',
      count: 806,
    );

    test(
      'should be a subclass of Year entity',
      () async {
        // assert
        expect(tYearModel, isA<Year>());
      },
    );

    group('fromJson', () {
      test(
        'should return a valid attribute when the JSON is valid',
        () async {
          // arrange
          final Map<String, dynamic> jsonMap =
              json.decode('{ "label": "2019", "count": 806 }');
          // act
          final result = YearModel.fromJson(jsonMap);
          // assert
          expect(result, tYearModel);
        },
      );
    });

    group('toJson', () {
      test(
        'should return a JSON map containing the proper data',
        () async {
          // act
          final result = tYearModel.toJson();
          // assert
          final expectedMap = {'label': '2019', 'count': 806};
          expect(result, expectedMap);
        },
      );
    });

    group('toJson and then fromJson', () {
      test('should convert all non-null options', () {
        // arrange
        final model = YearModel(label: '2003', count: 138);
        // act
        final result = YearModel.fromJson(model.toJson());
        // assert
        expect(result, equals(model));
      });
    });
  });
}
