//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter_test/flutter_test.dart';
import 'package:tanuki/features/modify/preso/validators/media_type.dart';

void main() {
  group('normal cases', () {
    test(
      'should accept common media type strings',
      () {
        expect(validateMediaType('image/jpeg'), null);
        expect(validateMediaType('application/xhtml+xml'), null);
        expect(validateMediaType('text/plain;charset=UTF-8'), null);
        expect(validateMediaType('application/octet-stream'), null);
        expect(validateMediaType('video/mp4'), null);
      },
    );
  });

  group('error cases', () {
    test(
      'should reject input with whitespace',
      () {
        expect(
          validateMediaType('image/ jpeg'),
          'Media type must not contain white space',
        );
      },
    );

    test(
      'should reject input with two slashes',
      () {
        expect(
          validateMediaType('image/jpeg/png'),
          'Media type may have only one slash (/)',
        );
      },
    );

    test(
      'should reject input with empty parts',
      () {
        expect(
          validateMediaType('/jpeg'),
          'Media type must have a type and subtype',
        );
        expect(
          validateMediaType('image/'),
          'Media type must have a type and subtype',
        );
      },
    );

    test(
      'should reject input with no slashes',
      () {
        expect(
          validateMediaType('image-jpeg'),
          'Media type must contain a slash (/)',
        );
      },
    );
  });
}
