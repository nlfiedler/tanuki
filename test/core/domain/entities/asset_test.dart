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

    test(
      'should implement from',
      () {
        expect(
          const AssetLocation(
            label: Some('classical garden'),
            city: None(),
            region: None(),
          ),
          AssetLocation.from("classical garden"),
        );
        expect(
          const AssetLocation(
            label: None(),
            city: Some('Portland'),
            region: Some('Oregon'),
          ),
          AssetLocation.from("Portland, Oregon"),
        );
        expect(
          const AssetLocation(
            label: Some('foo, bar, baz'),
            city: None(),
            region: None(),
          ),
          AssetLocation.from("foo, bar, baz"),
        );
        expect(
          const AssetLocation(
            label: Some('foo - bar - baz, quux'),
            city: None(),
            region: None(),
          ),
          AssetLocation.from("foo - bar - baz, quux"),
        );
        expect(
          const AssetLocation(
            label: Some('classical garden'),
            city: Some('Portland'),
            region: Some('Oregon'),
          ),
          AssetLocation.from("classical garden - Portland, Oregon"),
        );
        expect(
          const AssetLocation(
            label: Some('museum'),
            city: Some('Portland'),
            region: Some('Oregon'),
          ),
          AssetLocation.from("museum-Portland,Oregon"),
        );
        expect(
          const AssetLocation(
            label: None(),
            city: Some('A'),
            region: Some('B'),
          ),
          AssetLocation.from("a, b"),
        );
        expect(
          const AssetLocation(
            label: None(),
            city: Some('Castro Valley'),
            region: Some('California'),
          ),
          AssetLocation.from("castro valley, california"),
        );
      },
    );
  });
}
