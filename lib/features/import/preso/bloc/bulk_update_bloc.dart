//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:async';
import 'package:bloc/bloc.dart';
import 'package:equatable/equatable.dart';
import 'package:tanuki/core/domain/entities/input.dart';
import 'package:tanuki/core/domain/usecases/bulk_update.dart';

//
// events
//

abstract class BulkUpdateEvent extends Equatable {
  @override
  List<Object> get props => [];
}

class SubmitUpdates extends BulkUpdateEvent {
  final List<AssetInputId> inputs;

  SubmitUpdates({required this.inputs});
}

//
// states
//

abstract class BulkUpdateState extends Equatable {
  @override
  List<Object> get props => [];
}

class Initial extends BulkUpdateState {}

class Processing extends BulkUpdateState {}

class Finished extends BulkUpdateState {
  final int count;

  Finished({required this.count});

  @override
  List<Object> get props => [count];
}

class Error extends BulkUpdateState {
  final String message;

  Error({required this.message});

  @override
  List<Object> get props => [message];
}

//
// bloc
//

class BulkUpdateBloc extends Bloc<BulkUpdateEvent, BulkUpdateState> {
  final BulkUpdate usecase;

  BulkUpdateBloc({required this.usecase}) : super(Initial());

  @override
  Stream<BulkUpdateState> mapEventToState(
    BulkUpdateEvent event,
  ) async* {
    if (event is SubmitUpdates) {
      yield Processing();
      final result = await usecase(Params(assets: event.inputs));
      yield result.mapOrElse(
        (count) => Finished(count: count),
        (failure) => Error(message: failure.toString()),
      );
    }
  }
}
