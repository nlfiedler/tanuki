//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:async';
import 'package:bloc/bloc.dart';
import 'package:equatable/equatable.dart';
import 'package:meta/meta.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/core/domain/usecases/get_all_tags.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';

//
// events
//

abstract class AllTagsEvent extends Equatable {
  @override
  List<Object> get props => [];
}

class LoadAllTags extends AllTagsEvent {}

//
// states
//

abstract class AllTagsState extends Equatable {
  @override
  List<Object> get props => [];
}

class Empty extends AllTagsState {}

class Loading extends AllTagsState {}

class Loaded extends AllTagsState {
  final List<Tag> tags;

  Loaded({@required this.tags});

  @override
  List<Object> get props => [tags];
}

class Error extends AllTagsState {
  final String message;

  Error({@required this.message});

  @override
  List<Object> get props => [message];
}

//
// bloc
//

class AllTagsBloc extends Bloc<AllTagsEvent, AllTagsState> {
  final GetAllTags usecase;

  AllTagsBloc({this.usecase}) : super(Empty());

  @override
  Stream<AllTagsState> mapEventToState(
    AllTagsEvent event,
  ) async* {
    if (event is LoadAllTags) {
      yield Loading();
      final result = await usecase(NoParams());
      yield result.mapOrElse(
        (tags) => Loaded(tags: tags),
        (failure) => Error(message: failure.toString()),
      );
    }
  }
}
