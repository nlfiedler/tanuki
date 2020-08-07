//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
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
}
