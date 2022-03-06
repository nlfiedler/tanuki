//
// Copyright (c) 2022 Nathan Fiedler
//
import 'package:bloc/bloc.dart';
import 'package:equatable/equatable.dart';
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

  SubmitUpdate({required this.input});
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

  Finished({required this.asset});

  @override
  List<Object> get props => [asset];
}

class Error extends UpdateAssetState {
  final String message;

  Error({required this.message});

  @override
  List<Object> get props => [message];
}

//
// bloc
//

class UpdateAssetBloc extends Bloc<UpdateAssetEvent, UpdateAssetState> {
  final UpdateAsset usecase;

  UpdateAssetBloc({required this.usecase}) : super(Initial()) {
    on<SubmitUpdate>((event, emit) async {
      emit(Processing());
      final result = await usecase(Params(asset: event.input));
      emit(result.mapOrElse(
        (asset) => Finished(asset: asset),
        (failure) => Error(message: failure.toString()),
      ));
    });
  }
}
