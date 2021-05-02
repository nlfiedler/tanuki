//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:async';
import 'package:bloc/bloc.dart';
import 'package:equatable/equatable.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/usecases/get_all_locations.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';

//
// events
//

abstract class AllLocationsEvent extends Equatable {
  @override
  List<Object> get props => [];
}

class LoadAllLocations extends AllLocationsEvent {}

//
// states
//

abstract class AllLocationsState extends Equatable {
  @override
  List<Object> get props => [];
}

class Empty extends AllLocationsState {}

class Loading extends AllLocationsState {}

class Loaded extends AllLocationsState {
  final List<Location> locations;

  Loaded({required this.locations});

  @override
  List<Object> get props => [locations];
}

class Error extends AllLocationsState {
  final String message;

  Error({required this.message});

  @override
  List<Object> get props => [message];
}

//
// bloc
//

class AllLocationsBloc extends Bloc<AllLocationsEvent, AllLocationsState> {
  final GetAllLocations usecase;

  AllLocationsBloc({required this.usecase}) : super(Empty());

  @override
  Stream<AllLocationsState> mapEventToState(
    AllLocationsEvent event,
  ) async* {
    if (event is LoadAllLocations) {
      yield Loading();
      final result = await usecase(NoParams());
      yield result.mapOrElse(
        (locations) => Loaded(locations: locations),
        (failure) => Error(message: failure.toString()),
      );
    }
  }
}
