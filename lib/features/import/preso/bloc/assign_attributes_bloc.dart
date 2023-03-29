//
// Copyright (c) 2023 Nathan Fiedler
//
import 'package:bloc/bloc.dart';
import 'package:equatable/equatable.dart';

//
// events
//

abstract class AssignAttributesEvent extends Equatable {
  @override
  List<Object> get props => [];
}

class AssignTags extends AssignAttributesEvent {
  final List<String> tags;

  AssignTags({required this.tags});
}

class AssignLocation extends AssignAttributesEvent {
  final String? location;

  AssignLocation({required this.location});
}

class ToggleAsset extends AssignAttributesEvent {
  final String assetId;

  ToggleAsset({required this.assetId});
}

//
// states
//

class AssignAttributesState extends Equatable {
  // list of tags to be applied to the selected assets
  final List<String> tags;
  // location to be applied to the selected assets
  final String? location;
  // list of identifiers of the selected assets
  final Set<String> assets;

  AssignAttributesState(
      {List<String>? tags, this.location, Set<String>? assets})
      : tags = List.unmodifiable(tags ?? []),
        assets = Set.unmodifiable(assets ?? []);

  @override
  List<Object> get props => [tags, assets, location ?? ''];

  bool get submittable =>
      (tags.isNotEmpty || location != null) && assets.isNotEmpty;
}

//
// bloc
//

class AssignAttributesBloc
    extends Bloc<AssignAttributesEvent, AssignAttributesState> {
  List<String>? tags;
  String? location;
  final Set<String> assets = {};

  AssignAttributesBloc() : super(AssignAttributesState()) {
    on<ToggleAsset>((event, emit) {
      if (assets.contains(event.assetId)) {
        assets.remove(event.assetId);
      } else {
        assets.add(event.assetId);
      }
      emit(AssignAttributesState(
          tags: tags, location: location, assets: assets));
    });
    on<AssignTags>((event, emit) {
      tags = List.of(event.tags, growable: false);
      emit(AssignAttributesState(
          tags: tags, location: location, assets: assets));
    });
    on<AssignLocation>((event, emit) {
      location = event.location;
      emit(AssignAttributesState(
          tags: tags, location: location, assets: assets));
    });
  }
}
