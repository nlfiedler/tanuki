//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:async';
import 'package:bloc/bloc.dart';
import 'package:equatable/equatable.dart';
import 'package:meta/meta.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/core/domain/usecases/query_assets.dart';

//
// events
//

abstract class AssetBrowserEvent extends Equatable {
  @override
  List<Object> get props => [];
}

class LoadInitialAssets extends AssetBrowserEvent {}

class SelectTags extends AssetBrowserEvent {
  final List<String> tags;

  SelectTags({
    @required this.tags,
  });
}

class SelectLocations extends AssetBrowserEvent {
  final List<String> locations;

  SelectLocations({
    @required this.locations,
  });
}

class ToggleYear extends AssetBrowserEvent {
  final int year;

  ToggleYear({
    @required this.year,
  });
}

class ShowPage extends AssetBrowserEvent {
  final int page;

  ShowPage({
    @required this.page,
  });
}

class SetPageSize extends AssetBrowserEvent {
  final int size;

  SetPageSize({
    @required this.size,
  });
}

//
// states
//

abstract class AssetBrowserState extends Equatable {
  @override
  List<Object> get props => [];
}

class Empty extends AssetBrowserState {}

class Loading extends AssetBrowserState {}

class Loaded extends AssetBrowserState {
  final QueryResults results;
  final List<String> selectedTags;
  final List<String> selectedLocations;
  final Option<int> selectedYear;
  final int pageSize;
  final int pageNumber;
  final int lastPage;

  Loaded({
    @required this.results,
    @required this.pageNumber,
    @required tags,
    @required locations,
    @required this.selectedYear,
    @required this.lastPage,
    @required this.pageSize,
  })  : selectedTags = List.unmodifiable(tags),
        selectedLocations = List.unmodifiable(locations);

  @override
  List<Object> get props => [
        results,
        selectedTags,
        selectedLocations,
        selectedYear,
        pageNumber,
      ];
}

class Error extends AssetBrowserState {
  final String message;

  Error({@required this.message});

  @override
  List<Object> get props => [message];
}

//
// bloc
//

class AssetBrowserBloc extends Bloc<AssetBrowserEvent, AssetBrowserState> {
  final QueryAssets usecase;
  List<String> tags = [];
  List<String> locations = [];
  int pageSize = 18;
  int pageNumber = 1;
  Option<int> selectedYear = const None();

  AssetBrowserBloc({this.usecase}) : super(Empty());

  @override
  Stream<AssetBrowserState> mapEventToState(
    AssetBrowserEvent event,
  ) async* {
    if (event is LoadInitialAssets) {
      yield* _loadAssets();
    } else if (event is SelectTags) {
      if (state is Loaded) {
        tags = event.tags;
        pageNumber = 1;
        yield* _loadAssets();
      }
    } else if (event is SelectLocations) {
      if (state is Loaded) {
        locations = event.locations;
        pageNumber = 1;
        yield* _loadAssets();
      }
    } else if (event is ToggleYear) {
      if (state is Loaded) {
        _toggleYear(event.year);
        yield* _loadAssets();
      }
    } else if (event is ShowPage) {
      if (state is Loaded) {
        pageNumber = event.page;
        yield* _loadAssets();
      }
    } else if (event is SetPageSize) {
      pageSize = event.size;
      if (state is Loaded) {
        yield* _loadAssets();
      }
    }
  }

  void _toggleYear(int year) {
    selectedYear = selectedYear.mapOrElse(
      (value) => value == year ? None() : Some(year),
      () => Some(year),
    );
    pageNumber = 1;
  }

  Stream<AssetBrowserState> _loadAssets() async* {
    yield Loading();
    final after = selectedYear.map((t) => DateTime.utc(t, 1, 1));
    final before = selectedYear.map((t) => DateTime.utc(t + 1, 1, 1));
    final params = SearchParams(
      tags: tags,
      locations: locations,
      after: after,
      before: before,
    );
    final offset = pageSize * (pageNumber - 1);
    final result = await usecase(Params(
      params: params,
      count: pageSize,
      offset: offset,
    ));
    yield result.mapOrElse(
      (results) {
        final lastPage = (results.count / pageSize).ceil();
        return Loaded(
          results: results,
          pageNumber: lastPage > 0 ? pageNumber : 0,
          tags: tags,
          locations: locations,
          selectedYear: selectedYear,
          lastPage: lastPage,
          pageSize: pageSize,
        );
      },
      (failure) => Error(message: failure.toString()),
    );
  }
}
