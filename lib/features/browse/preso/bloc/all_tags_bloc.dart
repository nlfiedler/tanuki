//
// Copyright (c) 2022 Nathan Fiedler
//
import 'package:bloc/bloc.dart';
import 'package:equatable/equatable.dart';
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

  Loaded({required this.tags});

  @override
  List<Object> get props => [tags];
}

class Error extends AllTagsState {
  final String message;

  Error({required this.message});

  @override
  List<Object> get props => [message];
}

//
// bloc
//

class AllTagsBloc extends Bloc<AllTagsEvent, AllTagsState> {
  final GetAllTags usecase;

  AllTagsBloc({required this.usecase}) : super(Empty()) {
    on<LoadAllTags>((event, emit) async {
      emit(Loading());
      final result = await usecase(NoParams());
      emit(result.mapOrElse(
        (tags) => Loaded(tags: tags),
        (failure) => Error(message: failure.toString()),
      ));
    });
  }
}
