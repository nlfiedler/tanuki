//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:async';
import 'package:bloc/bloc.dart';
import 'package:equatable/equatable.dart';
import 'package:tanuki/core/domain/usecases/get_asset_count.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';

//
// events
//

abstract class AssetCountEvent extends Equatable {
  @override
  List<Object> get props => [];
}

class LoadAssetCount extends AssetCountEvent {}

//
// states
//

abstract class AssetCountState extends Equatable {
  @override
  List<Object> get props => [];
}

class Empty extends AssetCountState {}

class Loading extends AssetCountState {}

class Loaded extends AssetCountState {
  final int count;

  Loaded({required this.count});

  @override
  List<Object> get props => [count];
}

class Error extends AssetCountState {
  final String message;

  Error({required this.message});

  @override
  List<Object> get props => [message];
}

//
// bloc
//

class AssetCountBloc extends Bloc<AssetCountEvent, AssetCountState> {
  final GetAssetCount usecase;

  AssetCountBloc({required this.usecase}) : super(Empty());

  @override
  Stream<AssetCountState> mapEventToState(
    AssetCountEvent event,
  ) async* {
    if (event is LoadAssetCount) {
      yield Loading();
      final result = await usecase(NoParams());
      yield result.mapOrElse(
        (count) => Loaded(count: count),
        (failure) => Error(message: failure.toString()),
      );
    }
  }
}
