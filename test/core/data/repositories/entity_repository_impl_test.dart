//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:tanuki/core/data/models/attributes_model.dart';
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
}
