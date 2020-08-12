//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:async';
import 'package:bloc/bloc.dart';
import 'package:equatable/equatable.dart';
import 'package:meta/meta.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/core/domain/usecases/query_assets.dart';

//
// events
//

abstract class QueryAssetsEvent extends Equatable {
  @override
  List<Object> get props => [];
}

class LoadQueryAssets extends QueryAssetsEvent {
  final SearchParams params;
  final int count;
  final int offset;

  LoadQueryAssets({
    @required this.params,
    @required this.count,
    @required this.offset,
  });
}

//
// states
//

abstract class QueryAssetsState extends Equatable {
  @override
  List<Object> get props => [];
}

class Empty extends QueryAssetsState {}

class Loading extends QueryAssetsState {}

class Loaded extends QueryAssetsState {
  final QueryResults results;

  Loaded({@required this.results});

  @override
  List<Object> get props => [results];
}

class Error extends QueryAssetsState {
  final String message;

  Error({@required this.message});

  @override
  List<Object> get props => [message];
}

//
// bloc
//

class QueryAssetsBloc extends Bloc<QueryAssetsEvent, QueryAssetsState> {
  final QueryAssets usecase;

  QueryAssetsBloc({this.usecase}) : super(Empty());

  @override
  Stream<QueryAssetsState> mapEventToState(
    QueryAssetsEvent event,
  ) async* {
    if (event is LoadQueryAssets) {
      yield Loading();
      final result = await usecase(Params(
        params: event.params,
        offset: event.offset,
        count: event.count,
      ));
      yield result.mapOrElse(
        (results) => Loaded(results: results),
        (failure) => Error(message: failure.toString()),
      );
    }
  }
}
