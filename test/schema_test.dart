//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:test/test.dart';
import 'package:tanuki/schema.dart';

void main() {
  test('Query asset count via GraphQL', () async {
    var result = await graphql.parseAndExecute('{ count }');
    expect(result, {'count': 1234});
  });

  test('Query asset details using identifier', () async {
    var result = await graphql.parseAndExecute(
        '{ asset(id: "foobar") { filename filesize mimetype } }');
    // filesize is a BigInt which returns as a string
    expect(result, {
      'asset': {
        'filename': 'example.jpg',
        'filesize': '1048576',
        'mimetype': 'image/jpeg'
      }
    });
  });

  test('Query known asset tags via GraphQL', () async {
    var result = await graphql.parseAndExecute('{ tags { value count } }');
    expect(result, {
      'tags': [
        {'value': 'cat', 'count': 10}
      ]
    });
  });
}
