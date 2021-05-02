//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:typed_data';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:mockito/annotations.dart';
import 'package:tanuki/core/error/exceptions.dart';
import 'package:tanuki/core/error/failures.dart';
import 'package:tanuki/core/data/sources/asset_remote_data_source.dart';
import 'package:tanuki/core/data/repositories/asset_repository_impl.dart';
import './asset_repository_impl_test.mocks.dart';

@GenerateMocks([AssetRemoteDataSource])
void main() {
  late AssetRepositoryImpl repository;
  late MockAssetRemoteDataSource mockRemoteDataSource;

  setUp(() {
    mockRemoteDataSource = MockAssetRemoteDataSource();
    repository = AssetRepositoryImpl(
      remoteDataSource: mockRemoteDataSource,
    );
  });

  group('ingestAssets', () {
    test(
      'should return remote data when remote data source returns data',
      () async {
        // arrange
        when(mockRemoteDataSource.ingestAssets()).thenAnswer((_) async => 123);
        // act
        final result = await repository.ingestAssets();
        // assert
        verify(mockRemoteDataSource.ingestAssets());
        expect(result.unwrap(), isA<int>());
        expect(result.unwrap(), equals(123));
      },
    );

    test(
      'should return failure when remote data source is unsuccessful',
      () async {
        // arrange
        when(mockRemoteDataSource.ingestAssets()).thenThrow(ServerException());
        // act
        final result = await repository.ingestAssets();
        // assert
        verify(mockRemoteDataSource.ingestAssets());
        expect(result.err().unwrap(), isA<ServerFailure>());
      },
    );
  });

  group('uploadAsset', () {
    test(
      'should return remote data when remote data source returns data',
      () async {
        // arrange
        when(mockRemoteDataSource.uploadAsset(any))
            .thenAnswer((_) async => 'asset123');
        // act
        final result = await repository.uploadAsset('happy_cow.jpg');
        // assert
        verify(mockRemoteDataSource.uploadAsset('happy_cow.jpg'));
        expect(result.unwrap(), isA<String>());
        expect(result.unwrap(), equals('asset123'));
      },
    );

    test(
      'should return failure when remote data source is unsuccessful',
      () async {
        // arrange
        when(mockRemoteDataSource.uploadAsset(any))
            .thenThrow(ServerException());
        // act
        final result = await repository.uploadAsset('happy_cow.jpg');
        // assert
        verify(mockRemoteDataSource.uploadAsset('happy_cow.jpg'));
        expect(result.err().unwrap(), isA<ServerFailure>());
      },
    );
  });

  group('uploadAssetBytes', () {
    test(
      'should return remote data when remote data source returns data',
      () async {
        // arrange
        when(mockRemoteDataSource.uploadAssetBytes(any, any))
            .thenAnswer((_) async => 'asset123');
        // act
        final bytes = Uint8List(0);
        final result = await repository.uploadAssetBytes('happy.jpg', bytes);
        // assert
        verify(mockRemoteDataSource.uploadAssetBytes('happy.jpg', bytes));
        expect(result.unwrap(), isA<String>());
        expect(result.unwrap(), equals('asset123'));
      },
    );

    test(
      'should return failure when remote data source is unsuccessful',
      () async {
        // arrange
        when(mockRemoteDataSource.uploadAssetBytes(any, any))
            .thenThrow(ServerException());
        // act
        final bytes = Uint8List(0);
        final result = await repository.uploadAssetBytes('happy.jpg', bytes);
        // assert
        verify(mockRemoteDataSource.uploadAssetBytes('happy.jpg', bytes));
        expect(result.err().unwrap(), isA<ServerFailure>());
      },
    );
  });
}
