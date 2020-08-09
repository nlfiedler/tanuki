//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:convert';
import 'package:graphql/client.dart' as gql;
import 'package:http/http.dart' as http;
import 'package:mockito/mockito.dart';
import 'package:tanuki/core/data/models/attributes_model.dart';
import 'package:tanuki/core/data/sources/entity_remote_data_source.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tanuki/core/error/exceptions.dart';

class MockHttpClient extends Mock implements http.Client {}

void main() {
  EntityRemoteDataSourceImpl dataSource;
  MockHttpClient mockHttpClient;

  setUp(() {
    mockHttpClient = MockHttpClient();
    final link = gql.HttpLink(
      uri: 'http://example.com',
      httpClient: mockHttpClient,
    );
    final graphQLCient = gql.GraphQLClient(
      link: link,
      cache: gql.InMemoryCache(),
    );
    dataSource = EntityRemoteDataSourceImpl(client: graphQLCient);
  });

  void setUpMockHttpClientGraphQLError() {
    when(mockHttpClient.send(any)).thenAnswer((_) async {
      final response = {
        'data': null,
        'errors': [
          {
            'message': 'some kind of error occurred',
            'locations': [
              {'line': 2, 'column': 3}
            ],
            'path': ['beaten']
          }
        ]
      };
      final bytes = utf8.encode(json.encode(response));
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

  group('getAllLocations', () {
    void setUpMockHttpClientGraphQLResponse() {
      final response = {
        'data': {
          'locations': [
            {'label': 'tokyo', 'count': 806},
            {'label': 'paris', 'count': 269},
            {'label': 'london', 'count': 23},
          ]
        }
      };
      // graphql client uses the 'send' method
      when(mockHttpClient.send(any)).thenAnswer((_) async {
        final bytes = utf8.encode(json.encode(response));
        final stream = http.ByteStream.fromBytes(bytes);
        return http.StreamedResponse(stream, 200);
      });
    }

    test(
      'should return all of the locations',
      () async {
        // arrange
        setUpMockHttpClientGraphQLResponse();
        // act
        final result = await dataSource.getAllLocations();
        // assert
        expect(result, isA<List>());
        expect(result.length, equals(3));
        expect(
          result,
          containsAll(
            [
              LocationModel(label: 'tokyo', count: 806),
              LocationModel(label: 'paris', count: 269),
              LocationModel(label: 'london', count: 23),
            ],
          ),
        );
      },
    );

    test(
      'should report failure when response unsuccessful',
      () async {
        // arrange
        setUpMockHttpClientFailure403();
        // act, assert
        try {
          await dataSource.getAllLocations();
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
          await dataSource.getAllLocations();
          fail('should have raised an error');
        } catch (e) {
          expect(e, isA<ServerException>());
        }
      },
    );

    void setUpMockGraphQLNullResponse() {
      final response = {
        'data': {'locations': null}
      };
      // graphql client uses the 'send' method
      when(mockHttpClient.send(any)).thenAnswer((_) async {
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
        final result = await dataSource.getAllLocations();
        // assert
        expect(result, isNull);
      },
    );
  });

  group('getAllTags', () {
    void setUpMockHttpClientGraphQLResponse() {
      final response = {
        'data': {
          'tags': [
            {'label': 'kittens', 'count': 806},
            {'label': 'puppies', 'count': 269},
            {'label': 'birds', 'count': 23},
          ]
        }
      };
      // graphql client uses the 'send' method
      when(mockHttpClient.send(any)).thenAnswer((_) async {
        final bytes = utf8.encode(json.encode(response));
        final stream = http.ByteStream.fromBytes(bytes);
        return http.StreamedResponse(stream, 200);
      });
    }

    test(
      'should return all of the tags',
      () async {
        // arrange
        setUpMockHttpClientGraphQLResponse();
        // act
        final result = await dataSource.getAllTags();
        // assert
        expect(result, isA<List>());
        expect(result.length, equals(3));
        expect(
          result,
          containsAll(
            [
              TagModel(label: 'kittens', count: 806),
              TagModel(label: 'puppies', count: 269),
              TagModel(label: 'birds', count: 23),
            ],
          ),
        );
      },
    );

    test(
      'should report failure when response unsuccessful',
      () async {
        // arrange
        setUpMockHttpClientFailure403();
        // act, assert
        try {
          await dataSource.getAllTags();
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
          await dataSource.getAllTags();
          fail('should have raised an error');
        } catch (e) {
          expect(e, isA<ServerException>());
        }
      },
    );

    void setUpMockGraphQLNullResponse() {
      final response = {
        'data': {'tags': null}
      };
      // graphql client uses the 'send' method
      when(mockHttpClient.send(any)).thenAnswer((_) async {
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
        final result = await dataSource.getAllTags();
        // assert
        expect(result, isNull);
      },
    );
  });

  group('getAllYears', () {
    void setUpMockHttpClientGraphQLResponse() {
      final response = {
        'data': {
          'years': [
            {'label': '2019', 'count': 806},
            {'label': '2009', 'count': 269},
            {'label': '1999', 'count': 23},
          ]
        }
      };
      // graphql client uses the 'send' method
      when(mockHttpClient.send(any)).thenAnswer((_) async {
        final bytes = utf8.encode(json.encode(response));
        final stream = http.ByteStream.fromBytes(bytes);
        return http.StreamedResponse(stream, 200);
      });
    }

    test(
      'should return all of the years',
      () async {
        // arrange
        setUpMockHttpClientGraphQLResponse();
        // act
        final result = await dataSource.getAllYears();
        // assert
        expect(result, isA<List>());
        expect(result.length, equals(3));
        expect(
          result,
          containsAll(
            [
              YearModel(label: '2019', count: 806),
              YearModel(label: '2009', count: 269),
              YearModel(label: '1999', count: 23),
            ],
          ),
        );
      },
    );

    test(
      'should report failure when response unsuccessful',
      () async {
        // arrange
        setUpMockHttpClientFailure403();
        // act, assert
        try {
          await dataSource.getAllYears();
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
          await dataSource.getAllYears();
          fail('should have raised an error');
        } catch (e) {
          expect(e, isA<ServerException>());
        }
      },
    );

    void setUpMockGraphQLNullResponse() {
      final response = {
        'data': {'years': null}
      };
      // graphql client uses the 'send' method
      when(mockHttpClient.send(any)).thenAnswer((_) async {
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
        final result = await dataSource.getAllYears();
        // assert
        expect(result, isNull);
      },
    );
  });

  group('getAssetCount', () {
    void setUpMockHttpClientGraphQLResponse() {
      final response = {
        'data': {'count': 9413}
      };
      // graphql client uses the 'send' method
      when(mockHttpClient.send(any)).thenAnswer((_) async {
        final bytes = utf8.encode(json.encode(response));
        final stream = http.ByteStream.fromBytes(bytes);
        return http.StreamedResponse(stream, 200);
      });
    }

    test(
      'should return the asset count',
      () async {
        // arrange
        setUpMockHttpClientGraphQLResponse();
        // act
        final result = await dataSource.getAssetCount();
        // assert
        expect(result, equals(9413));
      },
    );

    test(
      'should report failure when response unsuccessful',
      () async {
        // arrange
        setUpMockHttpClientFailure403();
        // act, assert
        try {
          await dataSource.getAssetCount();
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
          await dataSource.getAssetCount();
          fail('should have raised an error');
        } catch (e) {
          expect(e, isA<ServerException>());
        }
      },
    );

    void setUpMockGraphQLNullResponse() {
      final response = {
        'data': {'count': null}
      };
      // graphql client uses the 'send' method
      when(mockHttpClient.send(any)).thenAnswer((_) async {
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
        final result = await dataSource.getAssetCount();
        // assert
        expect(result, isNull);
      },
    );
  });
}
