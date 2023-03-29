//
// Copyright (c) 2022 Nathan Fiedler
//
import 'dart:convert';
import 'package:flutter_test/flutter_test.dart';
import 'package:graphql/client.dart' as gql;
import 'package:http/http.dart' as http;
import 'package:mocktail/mocktail.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/data/models/asset_model.dart';
import 'package:tanuki/core/data/models/attributes_model.dart';
import 'package:tanuki/core/data/models/input_model.dart';
import 'package:tanuki/core/data/models/search_model.dart';
import 'package:tanuki/core/data/sources/entity_remote_data_source.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/core/error/exceptions.dart';

class MockHttpClient extends Mock implements http.Client {}

void main() {
  late EntityRemoteDataSourceImpl dataSource;
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
    dataSource = EntityRemoteDataSourceImpl(client: graphQLCient);
  });

  setUpAll(() {
    // mocktail needs a fallback for any() that involves custom types
    http.BaseRequest dummyRequest = http.Request(
      'GET',
      Uri(scheme: 'http', host: 'example.com', path: '/'),
    );
    registerFallbackValue(dummyRequest);
  });

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
    when(() => mockHttpClient.send(any())).thenAnswer((_) async {
      final bytes = List<int>.empty();
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
      when(() => mockHttpClient.send(any())).thenAnswer((_) async {
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
      when(() => mockHttpClient.send(any())).thenAnswer((_) async {
        final bytes = utf8.encode(json.encode(response));
        final stream = http.ByteStream.fromBytes(bytes);
        return http.StreamedResponse(stream, 200);
      });
    }

    test(
      'should return [] when response is null',
      () async {
        // arrange
        setUpMockGraphQLNullResponse();
        // act
        final result = await dataSource.getAllLocations();
        // assert
        expect(result, isA<List>());
        expect(result.length, equals(0));
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
      when(() => mockHttpClient.send(any())).thenAnswer((_) async {
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
      when(() => mockHttpClient.send(any())).thenAnswer((_) async {
        final bytes = utf8.encode(json.encode(response));
        final stream = http.ByteStream.fromBytes(bytes);
        return http.StreamedResponse(stream, 200);
      });
    }

    test(
      'should return [] when response is null',
      () async {
        // arrange
        setUpMockGraphQLNullResponse();
        // act
        final result = await dataSource.getAllTags();
        // assert
        expect(result, isA<List>());
        expect(result.length, equals(0));
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
      when(() => mockHttpClient.send(any())).thenAnswer((_) async {
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
      when(() => mockHttpClient.send(any())).thenAnswer((_) async {
        final bytes = utf8.encode(json.encode(response));
        final stream = http.ByteStream.fromBytes(bytes);
        return http.StreamedResponse(stream, 200);
      });
    }

    test(
      'should return [] when response is null',
      () async {
        // arrange
        setUpMockGraphQLNullResponse();
        // act
        final result = await dataSource.getAllYears();
        // assert
        expect(result, isA<List>());
        expect(result.length, equals(0));
      },
    );
  });

  group('getAsset', () {
    void setUpMockHttpClientGraphQLResponse() {
      final response = {
        'data': {
          'asset': {
            'id': 'asset123',
            'checksum': 'sha1-cafebabe',
            'filename': 'img_1234.jpg',
            'filesize': '1048576',
            'datetime': '2003-08-30T00:00:00.0+00:00',
            'mimetype': 'image/jpeg',
            'tags': ['clowns', 'snakes'],
            'userdate': null,
            'caption': '#snakes and #clowns are in my @batcave',
            'location': 'batcave'
          }
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
      'should return results of the query',
      () async {
        // arrange
        setUpMockHttpClientGraphQLResponse();
        // act
        final result = await dataSource.getAsset('asset123');
        // assert
        final expected = AssetModel(
          id: 'asset123',
          checksum: 'sha1-cafebabe',
          filename: 'img_1234.jpg',
          filesize: 1048576,
          datetime: DateTime.utc(2003, 8, 30),
          mimetype: 'image/jpeg',
          tags: const ['clowns', 'snakes'],
          userdate: const None(),
          caption: Some('#snakes and #clowns are in my @batcave'),
          location: Some('batcave'),
        );
        expect(result, equals(expected));
      },
    );

    test(
      'should report failure when response unsuccessful',
      () async {
        // arrange
        setUpMockHttpClientFailure403();
        // act, assert
        try {
          await dataSource.getAsset('asset123');
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
          await dataSource.getAsset('asset123');
          fail('should have raised an error');
        } catch (e) {
          expect(e, isA<ServerException>());
        }
      },
    );

    void setUpMockGraphQLNullResponse() {
      final response = {
        'data': {'search': null}
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
        final result = await dataSource.getAsset('asset123');
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
      when(() => mockHttpClient.send(any())).thenAnswer((_) async {
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
      when(() => mockHttpClient.send(any())).thenAnswer((_) async {
        final bytes = utf8.encode(json.encode(response));
        final stream = http.ByteStream.fromBytes(bytes);
        return http.StreamedResponse(stream, 200);
      });
    }

    test(
      'should return 0 when response is null',
      () async {
        // arrange
        setUpMockGraphQLNullResponse();
        // act
        final result = await dataSource.getAssetCount();
        // assert
        expect(result, equals(0));
      },
    );
  });

  group('queryAssets', () {
    void setUpMockHttpClientGraphQLResponse() {
      final response = {
        'data': {
          'search': {
            'results': [
              {
                'id': 'MjAyMC8wNS8yNC8x-mini-N5emVhamE4ajZuLmpwZw==',
                'filename': 'catmouse_1280p.jpg',
                'mimetype': 'image/jpeg',
                'location': 'outdoors',
                'datetime': '2020-05-24T18:02:15.0+00:00'
              }
            ],
            'count': 1
          }
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
      'should return results of the query',
      () async {
        // arrange
        setUpMockHttpClientGraphQLResponse();
        final params = SearchParams(tags: const ['mouse']);
        // act
        final result = await dataSource.queryAssets(params, 10, 0);
        // assert
        final expected = QueryResultsModel(
          results: [
            SearchResultModel(
              id: 'MjAyMC8wNS8yNC8x-mini-N5emVhamE4ajZuLmpwZw==',
              filename: 'catmouse_1280p.jpg',
              mimetype: 'image/jpeg',
              location: Some('outdoors'),
              datetime: DateTime.utc(2020, 5, 24, 18, 02, 15),
            )
          ],
          count: 1,
        );
        expect(result, equals(expected));
        expect(result!.results, equals(expected.results));
      },
    );

    test(
      'should report failure when response unsuccessful',
      () async {
        // arrange
        setUpMockHttpClientFailure403();
        final params = SearchParams(tags: const ['mouse']);
        // act, assert
        try {
          await dataSource.queryAssets(params, 10, 0);
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
        final params = SearchParams(tags: const ['mouse']);
        // act, assert
        try {
          await dataSource.queryAssets(params, 10, 0);
          fail('should have raised an error');
        } catch (e) {
          expect(e, isA<ServerException>());
        }
      },
    );

    void setUpMockGraphQLNullResponse() {
      final response = {
        'data': {'search': null}
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
        final params = SearchParams(tags: const ['mouse']);
        // act
        final result = await dataSource.queryAssets(params, 10, 0);
        // assert
        expect(result, isNull);
      },
    );
  });

  group('queryRecents', () {
    void setUpMockHttpClientGraphQLResponse() {
      final response = {
        'data': {
          'recent': {
            'results': [
              {
                'id': 'MjAyMC8wNS8yNC8x-mini-N5emVhamE4ajZuLmpwZw==',
                'filename': 'catmouse_1280p.jpg',
                'mimetype': 'image/jpeg',
                'location': 'outdoors',
                'datetime': '2020-05-24T18:02:15.0+00:00'
              }
            ],
            'count': 1
          }
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
      'should return results of the query',
      () async {
        // arrange
        setUpMockHttpClientGraphQLResponse();
        final Option<DateTime> since = Some(DateTime.now().toUtc());
        // act
        final result = await dataSource.queryRecents(
          since,
          const None(),
          const None(),
        );
        // assert
        final expected = QueryResultsModel(
          results: [
            SearchResultModel(
              id: 'MjAyMC8wNS8yNC8x-mini-N5emVhamE4ajZuLmpwZw==',
              filename: 'catmouse_1280p.jpg',
              mimetype: 'image/jpeg',
              location: Some('outdoors'),
              datetime: DateTime.utc(2020, 5, 24, 18, 02, 15),
            )
          ],
          count: 1,
        );
        expect(result, equals(expected));
        expect(result!.results, equals(expected.results));
      },
    );

    test(
      'should return results of the query if no time',
      () async {
        // arrange
        setUpMockHttpClientGraphQLResponse();
        const Option<DateTime> since = None();
        // act
        final result = await dataSource.queryRecents(
          since,
          const None(),
          const None(),
        );
        // assert
        final expected = QueryResultsModel(
          results: [
            SearchResultModel(
              id: 'MjAyMC8wNS8yNC8x-mini-N5emVhamE4ajZuLmpwZw==',
              filename: 'catmouse_1280p.jpg',
              mimetype: 'image/jpeg',
              location: Some('outdoors'),
              datetime: DateTime.utc(2020, 5, 24, 18, 02, 15),
            )
          ],
          count: 1,
        );
        expect(result, equals(expected));
        expect(result!.results, equals(expected.results));
      },
    );

    test(
      'should report failure when response unsuccessful',
      () async {
        // arrange
        setUpMockHttpClientFailure403();
        final Option<DateTime> since = Some(DateTime.now().toUtc());
        // act, assert
        try {
          await dataSource.queryRecents(
            since,
            const None(),
            const None(),
          );
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
        final Option<DateTime> since = Some(DateTime.now().toUtc());
        // act, assert
        try {
          await dataSource.queryRecents(
            since,
            const None(),
            const None(),
          );
          fail('should have raised an error');
        } catch (e) {
          expect(e, isA<ServerException>());
        }
      },
    );

    void setUpMockGraphQLNullResponse() {
      final response = {
        'data': {'recent': null}
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
        final Option<DateTime> since = Some(DateTime.now().toUtc());
        // act
        final result = await dataSource.queryRecents(
          since,
          const None(),
          const None(),
        );
        // assert
        expect(result, isNull);
      },
    );
  });

  group('bulkUpdate', () {
    void setUpMockHttpClientGraphQLResponse() {
      final response = {
        'data': {
          '__typename': 'Int',
          'bulkUpdate': 32,
        }
      };
      // graphql client uses the 'send' method
      when(() => mockHttpClient.send(any())).thenAnswer((_) async {
        final bytes = utf8.encode(json.encode(response));
        final stream = http.ByteStream.fromBytes(bytes);
        return http.StreamedResponse(stream, 200);
      });
    }

    final inputModel = AssetInputIdModel(
      id: 'asset123',
      input: AssetInputModel(
        tags: const ['clowns', 'snakes'],
        caption: Some('#snakes and #clowns are in my @batcave'),
        location: Some('batcave'),
        datetime: Some(DateTime.utc(2003, 8, 30)),
        mimetype: Some('image/jpeg'),
        filename: Some('img_1234.jpg'),
      ),
    );

    test(
      'should return results of the mutation',
      () async {
        // arrange
        setUpMockHttpClientGraphQLResponse();
        // act
        final result = await dataSource.bulkUpdate([inputModel]);
        // assert
        expect(result, equals(32));
      },
    );

    test(
      'should report failure when response unsuccessful',
      () async {
        // arrange
        setUpMockHttpClientFailure403();
        // act, assert
        try {
          await dataSource.bulkUpdate([inputModel]);
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
          await dataSource.bulkUpdate([inputModel]);
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
          'bulkUpdate': null,
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
      'should return 0 when response is null',
      () async {
        // arrange
        setUpMockGraphQLNullResponse();
        // act
        final result = await dataSource.bulkUpdate([inputModel]);
        // assert
        expect(result, equals(0));
      },
    );
  });

  group('updateAsset', () {
    void setUpMockHttpClientGraphQLResponse() {
      final response = {
        'data': {
          '__typename': 'Asset',
          'update': {
            '__typename': 'Asset',
            'id': 'asset123',
            'checksum': 'sha1-cafebabe',
            'filename': 'img_1234.jpg',
            'filesize': '1048576',
            'datetime': '2003-08-30T00:00:00.0+00:00',
            'mimetype': 'image/jpeg',
            'tags': ['clowns', 'snakes'],
            'userdate': null,
            'caption': '#snakes and #clowns are in my @batcave',
            'location': 'batcave'
          },
        }
      };
      // graphql client uses the 'send' method
      when(() => mockHttpClient.send(any())).thenAnswer((_) async {
        final bytes = utf8.encode(json.encode(response));
        final stream = http.ByteStream.fromBytes(bytes);
        return http.StreamedResponse(stream, 200);
      });
    }

    final inputModel = AssetInputIdModel(
      id: 'asset123',
      input: AssetInputModel(
        tags: const ['clowns', 'snakes'],
        caption: Some('#snakes and #clowns are in my @batcave'),
        location: Some('batcave'),
        datetime: Some(DateTime.utc(2003, 8, 30)),
        mimetype: Some('image/jpeg'),
        filename: Some('img_1234.jpg'),
      ),
    );

    test(
      'should return results of the mutation',
      () async {
        // arrange
        setUpMockHttpClientGraphQLResponse();
        // act
        final result = await dataSource.updateAsset(inputModel);
        // assert
        final expected = AssetModel(
          id: 'asset123',
          checksum: 'sha1-cafebabe',
          filename: 'img_1234.jpg',
          filesize: 1048576,
          datetime: DateTime.utc(2003, 8, 30),
          mimetype: 'image/jpeg',
          tags: const ['clowns', 'snakes'],
          userdate: const None(),
          caption: Some('#snakes and #clowns are in my @batcave'),
          location: Some('batcave'),
        );
        expect(result, equals(expected));
      },
    );

    test(
      'should report failure when response unsuccessful',
      () async {
        // arrange
        setUpMockHttpClientFailure403();
        // act, assert
        try {
          await dataSource.updateAsset(inputModel);
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
          await dataSource.updateAsset(inputModel);
          fail('should have raised an error');
        } catch (e) {
          expect(e, isA<ServerException>());
        }
      },
    );

    void setUpMockGraphQLNullResponse() {
      final response = {
        'data': {
          '__typename': 'Asset',
          'update': null,
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
        final result = await dataSource.updateAsset(inputModel);
        // assert
        expect(result, isNull);
      },
    );
  });
}
