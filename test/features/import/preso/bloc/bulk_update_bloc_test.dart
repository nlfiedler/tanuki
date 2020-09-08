//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/repositories/entity_repository.dart';
import 'package:tanuki/core/domain/usecases/bulk_update.dart';
import 'package:tanuki/core/error/failures.dart';
import 'package:tanuki/features/import/preso/bloc/bulk_update_bloc.dart';

class MockEntityRepository extends Mock implements EntityRepository {}

void main() {
  MockEntityRepository mockAssetRepository;
  BulkUpdate usecase;

  group('normal cases', () {
    setUp(() {
      mockAssetRepository = MockEntityRepository();
      usecase = BulkUpdate(mockAssetRepository);
      when(mockAssetRepository.bulkUpdate(any))
          .thenAnswer((_) async => Ok(101));
    });

    blocTest(
      'emits [] when nothing is added',
      build: () => BulkUpdateBloc(usecase: usecase),
      expect: [],
    );

    blocTest(
      'emits [Processing, Finished] when SubmitUpdates is added',
      build: () => BulkUpdateBloc(usecase: usecase),
      act: (bloc) => bloc.add(SubmitUpdates(inputs: [])),
      expect: [
        Processing(),
        Finished(count: 101),
      ],
    );
  });

  group('error cases', () {
    setUp(() {
      mockAssetRepository = MockEntityRepository();
      usecase = BulkUpdate(mockAssetRepository);
      when(mockAssetRepository.bulkUpdate(any))
          .thenAnswer((_) async => Err(ServerFailure('oh no!')));
    });

    blocTest(
      'emits [Uploading, Error] when repository returns an error',
      build: () => BulkUpdateBloc(usecase: usecase),
      act: (bloc) => bloc.add(SubmitUpdates(inputs: [])),
      expect: [
        Processing(),
        Error(message: 'ServerFailure(oh no!)'),
      ],
    );
  });
}
