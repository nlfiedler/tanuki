//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:flutter_test/flutter_test.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/asset.dart';

void main() {
  group('AssetLocation', () {
    test(
      'should implement description',
      () {
        // arrange
        const lcr = AssetLocation(
          label: Some('classical garden'),
          city: Some('Portland'),
          region: Some('Oregon'),
        );
        const cr = AssetLocation(
          label: None(),
          city: Some('Portland'),
          region: Some('Oregon'),
        );
        const r = AssetLocation(
          label: None(),
          city: None(),
          region: Some('Oregon'),
        );
        // act
        // assert
        expect(lcr.description(), 'classical garden - Portland, Oregon');
        expect(cr.description(), 'Portland, Oregon');
        expect(r.description(), 'Oregon');
      },
    );
  });
}
