//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:async';
import 'package:bloc/bloc.dart';
import 'package:equatable/equatable.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/usecases/get_all_years.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';

//
// events
//

abstract class AllYearsEvent extends Equatable {
  @override
  List<Object> get props => [];
}

class LoadAllYears extends AllYearsEvent {}

//
// states
//

abstract class AllYearsState extends Equatable {
  @override
  List<Object> get props => [];
}

class Empty extends AllYearsState {}

class Loading extends AllYearsState {}

class Loaded extends AllYearsState {
  final List<Year> years;

  Loaded({required this.years});

  @override
  List<Object> get props => [years];
}

class Error extends AllYearsState {
  final String message;

  Error({required this.message});

  @override
  List<Object> get props => [message];
}

//
// bloc
//

class AllYearsBloc extends Bloc<AllYearsEvent, AllYearsState> {
  final GetAllYears usecase;

  AllYearsBloc({required this.usecase}) : super(Empty());

  @override
  Stream<AllYearsState> mapEventToState(
    AllYearsEvent event,
  ) async* {
    if (event is LoadAllYears) {
      yield Loading();
      final result = await usecase(NoParams());
      yield result.mapOrElse(
        (years) => Loaded(years: years),
        (failure) => Error(message: failure.toString()),
      );
    }
  }
}
