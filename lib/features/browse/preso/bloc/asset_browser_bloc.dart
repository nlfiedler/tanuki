//
// Copyright (c) 2024 Nathan Fiedler
//
import 'dart:async';
import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:equatable/equatable.dart';
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
    required this.tags,
  });
}

class SelectLocations extends AssetBrowserEvent {
  final List<String> locations;

  SelectLocations({
    required this.locations,
  });
}

class SelectYear extends AssetBrowserEvent {
  final int? year;

  SelectYear({
    required this.year,
  });
}

/// Seasons akin to anime "seasons", just a convenient specifier, not at all
/// related to the weather or astronomy.
enum Season {
  /// January to March
  spring,

  /// April to June
  summer,

  /// July to September
  autumn,

  /// October to December
  winter,
}

class SelectSeason extends AssetBrowserEvent {
  final Season? season;

  SelectSeason({
    required this.season,
  });
}

class SetBeforeDate extends AssetBrowserEvent {
  final DateTime? date;

  SetBeforeDate({
    required this.date,
  });
}

class SetAfterDate extends AssetBrowserEvent {
  final DateTime? date;

  SetAfterDate({
    required this.date,
  });
}

class ShowPage extends AssetBrowserEvent {
  final int page;

  ShowPage({
    required this.page,
  });
}

class SetPageSize extends AssetBrowserEvent {
  final int size;

  SetPageSize({
    required this.size,
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
  final int? selectedYear;
  final Season? selectedSeason;
  final DateTime? beforeDate = null;
  final DateTime? afterDate = null;
  final int pageSize;
  final int pageNumber;
  final int lastPage;

  Loaded({
    required this.results,
    required this.pageNumber,
    required tags,
    required locations,
    required this.selectedYear,
    required this.selectedSeason,
    required this.lastPage,
    required this.pageSize,
  })  : selectedTags = List.unmodifiable(tags),
        selectedLocations = List.unmodifiable(locations);

  @override
  List<Object> get props => [
        results,
        selectedTags,
        selectedLocations,
        selectedYear ?? 0,
        pageNumber,
      ];
}

class Error extends AssetBrowserState {
  final String message;

  Error({required this.message});

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
  int? year;
  Season? season;
  int pageSize = 18;
  int pageNumber = 1;

  AssetBrowserBloc({required this.usecase}) : super(Empty()) {
    // enforce sequential ordering of event mapping due to the asynchronous
    // nature of this particular bloc
    on<AssetBrowserEvent>(_onEvent, transformer: sequential());
  }

  FutureOr<void> _onEvent(
    AssetBrowserEvent event,
    Emitter<AssetBrowserState> emit,
  ) async {
    if (event is LoadInitialAssets) {
      return _loadAssets(emit);
    } else if (event is SelectTags) {
      if (state is Loaded) {
        tags = event.tags;
        pageNumber = 1;
        return _loadAssets(emit);
      }
    } else if (event is SelectLocations) {
      if (state is Loaded) {
        locations = event.locations;
        pageNumber = 1;
        return _loadAssets(emit);
      }
    } else if (event is SelectYear) {
      if (state is Loaded) {
        year = event.year;
        pageNumber = 1;
        return _loadAssets(emit);
      }
    } else if (event is SelectSeason) {
      if (state is Loaded) {
        season = event.season;
        year ??= DateTime.now().year;
        pageNumber = 1;
        return _loadAssets(emit);
      }
    } else if (event is ShowPage) {
      if (state is Loaded) {
        pageNumber = event.page;
        return _loadAssets(emit);
      }
    } else if (event is SetPageSize) {
      pageSize = event.size;
      pageNumber = 1;
      if (state is Loaded) {
        return _loadAssets(emit);
      }
    }
  }

  Option<DateTime> getFirstDate() {
    if (year != null) {
      if (season != null) {
        switch (season) {
          case Season.summer:
            return Option.some(DateTime.utc(year!, 4, 1));
          case Season.autumn:
            return Option.some(DateTime.utc(year!, 7, 1));
          case Season.winter:
            return Option.some(DateTime.utc(year!, 10, 1));
          default:
            break;
        }
      }
      return Option.some(DateTime.utc(year!));
    }
    return const Option.none();
  }

  Option<DateTime> getLastDate() {
    if (year != null) {
      if (season != null) {
        switch (season) {
          case Season.spring:
            return Option.some(DateTime.utc(year!, 4, 1));
          case Season.summer:
            return Option.some(DateTime.utc(year!, 7, 1));
          case Season.autumn:
            return Option.some(DateTime.utc(year!, 10, 1));
          default:
            break;
        }
      }
      return Option.some(DateTime.utc(year! + 1));
    }
    return const Option.none();
  }

  SearchParams buildSearchParams() {
    // if all of the search fields are empty, then show all assets going back in
    // time from the current time
    final after = getFirstDate();
    final before = getLastDate();
    if (tags.isEmpty &&
        locations.isEmpty &&
        after.isNone() &&
        before.isNone()) {
      return SearchParams(
        before: Some(DateTime.now().toUtc()),
        sortOrder: const Some(SortOrder.descending),
      );
    } else {
      return SearchParams(
        tags: tags,
        locations: locations,
        after: getFirstDate(),
        before: getLastDate(),
      );
    }
  }

  Future<void> _loadAssets(Emitter<AssetBrowserState> emit) async {
    emit(Loading());
    final params = buildSearchParams();
    final offset = pageSize * (pageNumber - 1);
    final result = await usecase(Params(
      params: params,
      count: pageSize,
      offset: offset,
    ));
    emit(result.mapOrElse(
      (results) {
        final lastPage = (results.count / pageSize).ceil();
        return Loaded(
          results: results,
          pageNumber: lastPage > 0 ? pageNumber : 0,
          tags: tags,
          locations: locations,
          selectedYear: year,
          selectedSeason: season,
          lastPage: lastPage,
          pageSize: pageSize,
        );
      },
      (failure) => Error(message: failure.toString()),
    ));
  }
}
