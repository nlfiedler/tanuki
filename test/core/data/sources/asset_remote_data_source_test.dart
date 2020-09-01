//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:convert';
import 'dart:typed_data';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:mockito/mockito.dart';
import 'package:tanuki/core/data/sources/asset_remote_data_source.dart';
import 'package:tanuki/core/error/exceptions.dart';

class MockHttpClient extends Mock implements http.Client {}

const happyCowPath = '../tests/fixtures/dcp_1069.jpg';

void main() {
  AssetRemoteDataSource dataSource;
  MockHttpClient mockHttpClient;

  setUp(() {
    mockHttpClient = MockHttpClient();
    dataSource = AssetRemoteDataSourceImpl(
      client: mockHttpClient,
      baseUrl: 'http://example.com',
    );
  });

  void setUpMockHttpClientJsonError() {
    when(mockHttpClient.send(any)).thenAnswer((_) async {
      // empty response should be sufficiently wrong
      final bytes = List<int>();
      final stream = http.ByteStream.fromBytes(bytes);
      return http.StreamedResponse(stream, 200);
    });
  }

  void setUpMockHttpClientFailure403() {
    when(mockHttpClient.send(any)).thenAnswer((_) async {
      final bytes = List<int>();
      final stream = http.ByteStream.fromBytes(bytes);
      return http.StreamedResponse(stream, 403);
    });
  }

  group('uploadAsset', () {
    void setUpMockHttpClientJsonResponse() {
      final response = ['MjAyMC8wOC8yOS8wMzMw-mini-ZzAzczZiLmpwZw=='];
      when(mockHttpClient.send(any)).thenAnswer((_) async {
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
      when(mockHttpClient.send(any)).thenAnswer((_) async {
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
