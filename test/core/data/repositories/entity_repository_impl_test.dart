//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/data/models/attributes_model.dart';
import 'package:tanuki/core/data/models/search_model.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/core/error/exceptions.dart';
import 'package:tanuki/core/error/failures.dart';
import 'package:tanuki/core/data/sources/entity_remote_data_source.dart';
import 'package:tanuki/core/data/repositories/entity_repository_impl.dart';

class MockRemoteDataSource extends Mock implements EntityRemoteDataSource {}

void main() {
  EntityRepositoryImpl repository;
  MockRemoteDataSource mockRemoteDataSource;

  setUp(() {
    mockRemoteDataSource = MockRemoteDataSource();
    repository = EntityRepositoryImpl(
      remoteDataSource: mockRemoteDataSource,
    );
  });

  group('getAllLocations', () {
    test(
      'should return remote data when remote data source returns data',
      () async {
        // arrange
        final locations = [
          LocationModel(label: 'tokyo', count: 806),
          LocationModel(label: 'paris', count: 269),
          LocationModel(label: 'london', count: 23),
        ];
        when(mockRemoteDataSource.getAllLocations())
            .thenAnswer((_) async => locations);
        // act
        final result = await repository.getAllLocations();
        // assert
        verify(mockRemoteDataSource.getAllLocations());
        expect(result.unwrap(), isA<List>());
        expect(result.unwrap().length, equals(3));
        expect(result.unwrap(), containsAll(locations));
      },
    );

    test(
      'should return failure when remote data source returns null',
      () async {
        // arrange
        when(mockRemoteDataSource.getAllLocations())
            .thenAnswer((_) async => null);
        // act
        final result = await repository.getAllLocations();
        // assert
        verify(mockRemoteDataSource.getAllLocations());
        expect(result.err().unwrap(), isA<ServerFailure>());
      },
    );

    test(
      'should return failure when remote data source is unsuccessful',
      () async {
        // arrange
        when(mockRemoteDataSource.getAllLocations())
            .thenThrow(ServerException());
        // act
        final result = await repository.getAllLocations();
        // assert
        verify(mockRemoteDataSource.getAllLocations());
        expect(result.err().unwrap(), isA<ServerFailure>());
      },
    );
  });

  group('getAllTags', () {
    test(
      'should return remote data when remote data source returns data',
      () async {
        // arrange
        final tags = [
          TagModel(label: 'kittens', count: 806),
          TagModel(label: 'snakes', count: 269),
          TagModel(label: 'clouds', count: 23),
        ];
        when(mockRemoteDataSource.getAllTags()).thenAnswer((_) async => tags);
        // act
        final result = await repository.getAllTags();
        // assert
        verify(mockRemoteDataSource.getAllTags());
        expect(result.unwrap(), isA<List>());
        expect(result.unwrap().length, equals(3));
        expect(result.unwrap(), containsAll(tags));
      },
    );

    test(
      'should return failure when remote data source returns null',
      () async {
        // arrange
        when(mockRemoteDataSource.getAllTags()).thenAnswer((_) async => null);
        // act
        final result = await repository.getAllTags();
        // assert
        verify(mockRemoteDataSource.getAllTags());
        expect(result.err().unwrap(), isA<ServerFailure>());
      },
    );

    test(
      'should return failure when remote data source is unsuccessful',
      () async {
        // arrange
        when(mockRemoteDataSource.getAllTags()).thenThrow(ServerException());
        // act
        final result = await repository.getAllTags();
        // assert
        verify(mockRemoteDataSource.getAllTags());
        expect(result.err().unwrap(), isA<ServerFailure>());
      },
    );
  });

  group('getAllYears', () {
    test(
      'should return remote data when remote data source returns data',
      () async {
        // arrange
        final years = [
          YearModel(label: '2019', count: 806),
          YearModel(label: '2009', count: 269),
          YearModel(label: '1999', count: 23),
        ];
        when(mockRemoteDataSource.getAllYears()).thenAnswer((_) async => years);
        // act
        final result = await repository.getAllYears();
        // assert
        verify(mockRemoteDataSource.getAllYears());
        expect(result.unwrap(), isA<List>());
        expect(result.unwrap().length, equals(3));
        expect(result.unwrap(), containsAll(years));
      },
    );

    test(
      'should return failure when remote data source returns null',
      () async {
        // arrange
        when(mockRemoteDataSource.getAllYears()).thenAnswer((_) async => null);
        // act
        final result = await repository.getAllYears();
        // assert
        verify(mockRemoteDataSource.getAllYears());
        expect(result.err().unwrap(), isA<ServerFailure>());
      },
    );

    test(
      'should return failure when remote data source is unsuccessful',
      () async {
        // arrange
        when(mockRemoteDataSource.getAllYears()).thenThrow(ServerException());
        // act
        final result = await repository.getAllYears();
        // assert
        verify(mockRemoteDataSource.getAllYears());
        expect(result.err().unwrap(), isA<ServerFailure>());
      },
    );
  });

  group('getAssetCount', () {
    test(
      'should return remote data when remote data source returns data',
      () async {
        // arrange
        when(mockRemoteDataSource.getAssetCount())
            .thenAnswer((_) async => 9413);
        // act
        final result = await repository.getAssetCount();
        // assert
        verify(mockRemoteDataSource.getAssetCount());
        expect(result.unwrap(), equals(9413));
      },
    );

    test(
      'should return failure when remote data source returns null',
      () async {
        // arrange
        when(mockRemoteDataSource.getAssetCount())
            .thenAnswer((_) async => null);
        // act
        final result = await repository.getAssetCount();
        // assert
        verify(mockRemoteDataSource.getAssetCount());
        expect(result.err().unwrap(), isA<ServerFailure>());
      },
    );

    test(
      'should return failure when remote data source is unsuccessful',
      () async {
        // arrange
        when(mockRemoteDataSource.getAssetCount()).thenThrow(ServerException());
        // act
        final result = await repository.getAssetCount();
        // assert
        verify(mockRemoteDataSource.getAssetCount());
        expect(result.err().unwrap(), isA<ServerFailure>());
      },
    );
  });

  group('queryAssets', () {
    test(
      'should return remote data when remote data source returns data',
      () async {
        // arrange
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
        when(mockRemoteDataSource.queryAssets(any, any, any))
            .thenAnswer((_) async => expected);
        // act
        final params = SearchParams(tags: ["mouse"]);
        final result = await repository.queryAssets(params, 10, 0);
        // assert
        verify(mockRemoteDataSource.queryAssets(params, 10, 0));
        expect(result.unwrap(), equals(expected));
      },
    );

    test(
      'should return failure when remote data source returns null',
      () async {
        // arrange
        when(mockRemoteDataSource.queryAssets(any, any, any))
            .thenAnswer((_) async => null);
        // act
        final params = SearchParams(tags: ["mouse"]);
        final result = await repository.queryAssets(params, 10, 0);
        // assert
        verify(mockRemoteDataSource.queryAssets(params, 10, 0));
        expect(result.err().unwrap(), isA<ServerFailure>());
      },
    );

    test(
      'should return failure when remote data source is unsuccessful',
      () async {
        // arrange
        when(mockRemoteDataSource.queryAssets(any, any, any))
            .thenThrow(ServerException());
        // act
        final params = SearchParams(tags: ["mouse"]);
        final result = await repository.queryAssets(params, 10, 0);
        // assert
        verify(mockRemoteDataSource.queryAssets(params, 10, 0));
        expect(result.err().unwrap(), isA<ServerFailure>());
      },
    );
  });
}