//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:bloc/bloc.dart';
import 'package:equatable/equatable.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/domain/usecases/get_asset_locations.dart';
import 'package:tanuki/core/domain/usecases/usecase.dart';

//
// events
//

abstract class RawLocationsEvent extends Equatable {
  @override
  List<Object> get props => [];
}

class LoadRawLocations extends RawLocationsEvent {}

//
// states
//

abstract class RawLocationsState extends Equatable {
  @override
  List<Object> get props => [];
}

class Empty extends RawLocationsState {}

class Loading extends RawLocationsState {}

class Loaded extends RawLocationsState {
  final List<AssetLocation> locations;

  Loaded({required this.locations});

  @override
  List<Object> get props => [locations];
}

class Error extends RawLocationsState {
  final String message;

  Error({required this.message});

  @override
  List<Object> get props => [message];
}

//
// bloc
//

class RawLocationsBloc extends Bloc<RawLocationsEvent, RawLocationsState> {
  final GetAssetLocations usecase;

  RawLocationsBloc({required this.usecase}) : super(Empty()) {
    on<LoadRawLocations>((event, emit) async {
      emit(Loading());
      final result = await usecase(NoParams());
      emit(result.mapOrElse(
        (locations) => Loaded(locations: locations),
        (failure) => Error(message: failure.toString()),
      ));
    });
  }
}
