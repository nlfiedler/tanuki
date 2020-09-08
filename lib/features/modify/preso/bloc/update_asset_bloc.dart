//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:async';
import 'package:bloc/bloc.dart';
import 'package:equatable/equatable.dart';
import 'package:meta/meta.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/domain/entities/input.dart';
import 'package:tanuki/core/domain/usecases/update_asset.dart';

//
// events
//

abstract class UpdateAssetEvent extends Equatable {
  @override
  List<Object> get props => [];
}

class SubmitUpdate extends UpdateAssetEvent {
  final AssetInputId input;

  SubmitUpdate({@required this.input});
}

//
// states
//

abstract class UpdateAssetState extends Equatable {
  @override
  List<Object> get props => [];
}

class Initial extends UpdateAssetState {}

class Processing extends UpdateAssetState {}

class Finished extends UpdateAssetState {
  final Asset asset;

  Finished({@required this.asset});

  @override
  List<Object> get props => [asset];
}

class Error extends UpdateAssetState {
  final String message;

  Error({@required this.message});

  @override
  List<Object> get props => [message];
}

//
// bloc
//

class UpdateAssetBloc extends Bloc<UpdateAssetEvent, UpdateAssetState> {
  final UpdateAsset usecase;

  UpdateAssetBloc({this.usecase}) : super(Initial());

  @override
  Stream<UpdateAssetState> mapEventToState(
    UpdateAssetEvent event,
  ) async* {
    if (event is SubmitUpdate) {
      yield Processing();
      final result = await usecase(Params(asset: event.input));
      yield result.mapOrElse(
        (asset) => Finished(asset: asset),
        (failure) => Error(message: failure.toString()),
      );
    }
  }
}
