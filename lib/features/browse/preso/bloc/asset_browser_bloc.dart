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

class SelectDates extends AssetBrowserEvent {
  final List<DateTime> dates;

  SelectDates({
    @required this.dates,
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
  final List<DateTime> selectedDates;
  final int pageSize;
  final int pageNumber;
  final int lastPage;

  Loaded({
    @required this.results,
    @required this.pageNumber,
    @required tags,
    @required locations,
    @required dates,
    @required this.lastPage,
    @required this.pageSize,
  })  : selectedTags = List.unmodifiable(tags),
        selectedLocations = List.unmodifiable(locations),
        selectedDates = List.unmodifiable(dates);

  @override
  List<Object> get props => [
        results,
        selectedTags,
        selectedLocations,
        selectedDates,
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
  List<DateTime> dates = [];
  int pageSize = 18;
  int pageNumber = 1;

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
    } else if (event is SelectDates) {
      if (state is Loaded) {
        dates = event.dates;
        pageNumber = 1;
        yield* _loadAssets();
      }
    } else if (event is ShowPage) {
      if (state is Loaded) {
        pageNumber = event.page;
        yield* _loadAssets();
      }
    } else if (event is SetPageSize) {
      pageSize = event.size;
      pageNumber = 1;
      if (state is Loaded) {
        yield* _loadAssets();
      }
    }
  }

  Option<DateTime> getFirstDate(List<DateTime> dates) {
    if (dates.isNotEmpty) {
      return Some(dates[0].toUtc());
    }
    return None();
  }

  Option<DateTime> getLastDate(List<DateTime> dates) {
    if (dates.length == 2) {
      return Some(dates[1].toUtc());
    }
    return None();
  }

  Stream<AssetBrowserState> _loadAssets() async* {
    yield Loading();
    final params = SearchParams(
      tags: tags,
      locations: locations,
      after: getFirstDate(dates),
      before: getLastDate(dates),
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
          dates: dates,
          lastPage: lastPage,
          pageSize: pageSize,
        );
      },
      (failure) => Error(message: failure.toString()),
    );
  }
}
