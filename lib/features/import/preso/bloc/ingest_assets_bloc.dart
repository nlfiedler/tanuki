//
// Copyright (c) 2022 Nathan Fiedler
//
import 'package:bloc/bloc.dart';
import 'package:equatable/equatable.dart';
import 'package:tanuki/core/domain/usecases/ingest_assets.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';

//
// events
//

abstract class IngestAssetsEvent extends Equatable {
  @override
  List<Object> get props => [];
}

class ProcessUploads extends IngestAssetsEvent {}

//
// states
//

abstract class IngestAssetsState extends Equatable {
  @override
  List<Object> get props => [];
}

class Initial extends IngestAssetsState {}

class Processing extends IngestAssetsState {}

class Finished extends IngestAssetsState {
  final int count;

  Finished({required this.count});

  @override
  List<Object> get props => [count];
}

class Error extends IngestAssetsState {
  final String message;

  Error({required this.message});

  @override
  List<Object> get props => [message];
}

//
// bloc
//

class IngestAssetsBloc extends Bloc<IngestAssetsEvent, IngestAssetsState> {
  final IngestAssets usecase;

  IngestAssetsBloc({required this.usecase}) : super(Initial()) {
    on<ProcessUploads>((event, emit) async {
      emit(Processing());
      final result = await usecase(NoParams());
      emit(result.mapOrElse(
        (count) => Finished(count: count),
        (failure) => Error(message: failure.toString()),
      ));
    });
  }
}
