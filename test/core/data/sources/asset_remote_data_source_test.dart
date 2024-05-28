//
// Copyright (c) 2022 Nathan Fiedler
//
import 'dart:convert';
import 'dart:typed_data';
import 'package:flutter_test/flutter_test.dart';
import 'package:graphql/client.dart' as gql;
import 'package:http/http.dart' as http;
import 'package:mocktail/mocktail.dart';
import 'package:tanuki/core/data/sources/asset_remote_data_source.dart';
import 'package:tanuki/core/error/exceptions.dart';

class MockHttpClient extends Mock implements http.Client {}

const happyCowPath = 'tests/fixtures/dcp_1069.jpg';

void main() {
  late AssetRemoteDataSource dataSource;
  late MockHttpClient mockHttpClient;

  setUp(() {
    mockHttpClient = MockHttpClient();
    final link = gql.HttpLink(
      'http://example.com',
      httpClient: mockHttpClient,
    );
    final graphQLCient = gql.GraphQLClient(
      link: link,
      cache: gql.GraphQLCache(),
    );
    dataSource = AssetRemoteDataSourceImpl(
      httpClient: mockHttpClient,
      baseUrl: 'http://example.com',
      gqlClient: graphQLCient,
    );
  });

  setUpAll(() {
    // mocktail needs a fallback for any() that involves custom types
    http.BaseRequest dummyRequest = http.Request(
      'GET',
      Uri(scheme: 'http', host: 'example.com', path: '/'),
    );
    registerFallbackValue(dummyRequest);
  });

  void setUpMockHttpClientJsonError() {
    when(() => mockHttpClient.send(any())).thenAnswer((_) async {
      // empty response should be sufficiently wrong
      final bytes = List<int>.empty();
      final stream = http.ByteStream.fromBytes(bytes);
      return http.StreamedResponse(stream, 200);
    });
  }

  void setUpMockHttpClientGraphQLError() {
    when(() => mockHttpClient.send(any())).thenAnswer((_) async {
      final response = {
        'data': null,
        'errors': [
          {
            'message': 'some kind of error occurred',
            'locations': [
              {'line': 2, 'column': 3}
            ],
            'path': ['ingest']
          }
        ]
      };
      final bytes = utf8.encode(json.encode(response));
      final stream = http.ByteStream.fromBytes(bytes);
      return http.StreamedResponse(stream, 200);
    });
  }

  void setUpMockHttpClientFailure403() {
    when(() => mockHttpClient.send(any())).thenAnswer((_) async {
      final bytes = List<int>.empty();
      final stream = http.ByteStream.fromBytes(bytes);
      return http.StreamedResponse(stream, 403);
    });
  }

  group('ingestAssets', () {
    void setUpMockHttpClientGraphQLResponse() {
      final response = {
        'data': {
          '__typename': 'Int',
          'ingest': 101,
        }
      };
      // graphql client uses the 'send' method
      when(() => mockHttpClient.send(any())).thenAnswer((_) async {
        final bytes = utf8.encode(json.encode(response));
        final stream = http.ByteStream.fromBytes(bytes);
        return http.StreamedResponse(stream, 200);
      });
    }

    test(
      'should return number of ingested assets',
      () async {
        // arrange
        setUpMockHttpClientGraphQLResponse();
        // act
        final result = await dataSource.ingestAssets();
        // assert
        expect(result, equals(101));
      },
    );

    test(
      'should report failure when response unsuccessful',
      () async {
        // arrange
        setUpMockHttpClientFailure403();
        // act, assert
        try {
          await dataSource.ingestAssets();
          fail('should have raised an error');
        } catch (e) {
          expect(e, isA<ServerException>());
        }
      },
    );

    test(
      'should raise error when GraphQL server returns an error',
      () async {
        // arrange
        setUpMockHttpClientGraphQLError();
        // act, assert
        try {
          await dataSource.ingestAssets();
          fail('should have raised an error');
        } catch (e) {
          expect(e, isA<ServerException>());
        }
      },
    );

    void setUpMockGraphQLNullResponse() {
      final response = {
        'data': {
          '__typename': 'Int',
          'ingest': null,
        }
      };
      // graphql client uses the 'send' method
      when(() => mockHttpClient.send(any())).thenAnswer((_) async {
        final bytes = utf8.encode(json.encode(response));
        final stream = http.ByteStream.fromBytes(bytes);
        return http.StreamedResponse(stream, 200);
      });
    }

    test(
      'should return null when response is null',
      () async {
        // arrange
        setUpMockGraphQLNullResponse();
        // act
        final result = await dataSource.ingestAssets();
        // assert
        expect(result, equals(0));
      },
    );
  });

  group('replaceAssetBytes', () {
    void setUpMockHttpClientJsonResponse() {
      final response = ['MjAyMC8wOC8yOS8wMzMw-mini-ZzAzczZiLmpwZw=='];
      when(() => mockHttpClient.send(any())).thenAnswer((_) async {
        final bytes = utf8.encode(json.encode(response));
        final stream = http.ByteStream.fromBytes(bytes);
        return http.StreamedResponse(stream, 200);
      });
    }

    test(
      'should return new asset identifier on success',
      () async {
        // arrange
        setUpMockHttpClientJsonResponse();
        // act
        final result = await dataSource.replaceAssetBytes(
          'asset123',
          'filename.ext',
          Uint8List(0),
        );
        // assert
        expect(result, isA<String>());
        expect(result, equals('MjAyMC8wOC8yOS8wMzMw-mini-ZzAzczZiLmpwZw=='));
      },
    );

    test(
      'should raise error when server returns an error',
      () async {
        // arrange
        setUpMockHttpClientFailure403();
        // act, assert
        try {
          await dataSource.replaceAssetBytes(
            'asset123',
            'filename.ext',
            Uint8List(0),
          );
          fail('should have raised an error');
        } catch (e) {
          expect(e, isA<ServerException>());
        }
      },
    );

    test(
      'should report failure when response malformed',
      () async {
        // arrange
        setUpMockHttpClientJsonError();
        // act, assert
        try {
          await dataSource.replaceAssetBytes(
            'asset123',
            'filename.ext',
            Uint8List(0),
          );
          fail('should have raised an error');
        } catch (e) {
          expect(e, isA<ServerException>());
        }
      },
    );
  });

  group('uploadAsset', () {
    void setUpMockHttpClientJsonResponse() {
      final response = ['MjAyMC8wOC8yOS8wMzMw-mini-ZzAzczZiLmpwZw=='];
      when(() => mockHttpClient.send(any())).thenAnswer((_) async {
        final bytes = utf8.encode(json.encode(response));
        final stream = http.ByteStream.fromBytes(bytes);
        return http.StreamedResponse(stream, 200);
      });
    }

    test(
      'should return new asset identifier on success',
      () async {
        // arrange
        setUpMockHttpClientJsonResponse();
        // act
        final result = await dataSource.uploadAsset(happyCowPath);
        // assert
        expect(result, isA<String>());
        expect(result, equals('MjAyMC8wOC8yOS8wMzMw-mini-ZzAzczZiLmpwZw=='));
      },
    );

    test(
      'should raise error when server returns an error',
      () async {
        // arrange
        setUpMockHttpClientFailure403();
        // act, assert
        try {
          await dataSource.uploadAsset(happyCowPath);
          fail('should have raised an error');
        } catch (e) {
          expect(e, isA<ServerException>());
        }
      },
    );

    test(
      'should report failure when response malformed',
      () async {
        // arrange
        setUpMockHttpClientJsonError();
        // act, assert
        try {
          await dataSource.uploadAsset(happyCowPath);
          fail('should have raised an error');
        } catch (e) {
          expect(e, isA<ServerException>());
        }
      },
    );
  });

  group('uploadAssetBytes', () {
    void setUpMockHttpClientJsonResponse() {
      final response = ['MjAyMC8wOC8yOS8wMzMw-mini-ZzAzczZiLmpwZw=='];
      when(() => mockHttpClient.send(any())).thenAnswer((_) async {
        final bytes = utf8.encode(json.encode(response));
        final stream = http.ByteStream.fromBytes(bytes);
        return http.StreamedResponse(stream, 200);
      });
    }

    test(
      'should return new asset identifier on success',
      () async {
        // arrange
        setUpMockHttpClientJsonResponse();
        // act
        final result = await dataSource.uploadAssetBytes(
          'filename.ext',
          Uint8List(0),
        );
        // assert
        expect(result, isA<String>());
        expect(result, equals('MjAyMC8wOC8yOS8wMzMw-mini-ZzAzczZiLmpwZw=='));
      },
    );

    test(
      'should raise error when server returns an error',
      () async {
        // arrange
        setUpMockHttpClientFailure403();
        // act, assert
        try {
          await dataSource.uploadAssetBytes(
            'filename.ext',
            Uint8List(0),
          );
          fail('should have raised an error');
        } catch (e) {
          expect(e, isA<ServerException>());
        }
      },
    );

    test(
      'should report failure when response malformed',
      () async {
        // arrange
        setUpMockHttpClientJsonError();
        // act, assert
        try {
          await dataSource.uploadAssetBytes(
            'filename.ext',
            Uint8List(0),
          );
          fail('should have raised an error');
        } catch (e) {
          expect(e, isA<ServerException>());
        }
      },
    );
  });
}
