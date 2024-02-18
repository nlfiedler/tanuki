//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:flutter_test/flutter_test.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';

void main() {
  group('Location', () {
    test(
      'should implement Comparable',
      () {
        // arrange
        const first = Location(label: 'aaa', count: 10);
        const second = Location(label: 'bbb', count: 5);
        // act
        // assert
        expect(first.compareTo(second), -1);
        expect(second.compareTo(first), 1);
        expect(first.compareTo(first), 0);
      },
    );
  });

  group('Tag', () {
    test(
      'should implement Comparable',
      () {
        // arrange
        const first = Tag(label: 'aaa', count: 10);
        const second = Tag(label: 'bbb', count: 5);
        // act
        // assert
        expect(first.compareTo(second), -1);
        expect(second.compareTo(first), 1);
        expect(first.compareTo(first), 0);
      },
    );
  });

  group('Year', () {
    test(
      'should implement Comparable',
      () {
        // arrange
        final first = Year(label: '201', count: 10);
        final second = Year(label: '1001', count: 5);
        // act
        // assert
        expect(first.compareTo(second), -1);
        expect(second.compareTo(first), 1);
        expect(first.compareTo(first), 0);
      },
    );
  });
}
