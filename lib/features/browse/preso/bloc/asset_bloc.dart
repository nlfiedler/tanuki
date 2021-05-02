//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:async';
import 'package:bloc/bloc.dart';
import 'package:equatable/equatable.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/domain/usecases/get_asset.dart';

//
// events
//

abstract class AssetEvent extends Equatable {
  @override
  List<Object> get props => [];
}

class LoadAsset extends AssetEvent {
  final String id;

  LoadAsset({required this.id});
}

//
// states
//

abstract class AssetState extends Equatable {
  @override
  List<Object> get props => [];
}

class Empty extends AssetState {}

class Loading extends AssetState {}

class Loaded extends AssetState {
  final Asset asset;

  Loaded({required this.asset});

  @override
  List<Object> get props => [asset];
}

class Error extends AssetState {
  final String message;

  Error({required this.message});

  @override
  List<Object> get props => [message];
}

//
// bloc
//

class AssetBloc extends Bloc<AssetEvent, AssetState> {
  final GetAsset usecase;

  AssetBloc({required this.usecase}) : super(Empty());

  @override
  Stream<AssetState> mapEventToState(
    AssetEvent event,
  ) async* {
    if (event is LoadAsset) {
      yield Loading();
      final result = await usecase(Params(assetId: event.id));
      yield result.mapOrElse(
        (asset) => Loaded(asset: asset),
        (failure) => Error(message: failure.toString()),
      );
    }
  }
}
