//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:async';
import 'package:bloc/bloc.dart';
import 'package:equatable/equatable.dart';
import 'package:meta/meta.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/core/domain/usecases/query_recents.dart';

//
// events
//

abstract class RecentImportsEvent extends Equatable {
  @override
  List<Object> get props => [];
}

enum RecentTimeRange { day, week, month, ever }

extension RecentTimeRangeExt on RecentTimeRange {
  Option<DateTime> get asDate {
    final now = DateTime.now();
    switch (this) {
      case RecentTimeRange.day:
        return Some(now.subtract(Duration(days: 1)));
      case RecentTimeRange.week:
        return Some(now.subtract(Duration(days: 7)));
      case RecentTimeRange.month:
        return Some(now.subtract(Duration(days: 30)));
      case RecentTimeRange.ever:
        return None();
    }
    return None();
  }
}

class FindRecents extends RecentImportsEvent {
  final RecentTimeRange range;

  FindRecents({@required this.range});
}

/// Submit the query again for the same time range as before.
class RefreshResults extends RecentImportsEvent {}

//
// states
//

abstract class RecentImportsState extends Equatable {
  @override
  List<Object> get props => [];
}

class Empty extends RecentImportsState {}

class Loading extends RecentImportsState {}

class Loaded extends RecentImportsState {
  final QueryResults results;
  final RecentTimeRange range;

  Loaded({@required this.results, @required this.range});

  @override
  List<Object> get props => [range, results];
}

class Error extends RecentImportsState {
  final String message;

  Error({@required this.message});

  @override
  List<Object> get props => [message];
}

//
// bloc
//

class RecentImportsBloc extends Bloc<RecentImportsEvent, RecentImportsState> {
  final QueryRecents usecase;
  RecentTimeRange prevQueryRange = RecentTimeRange.day;

  RecentImportsBloc({this.usecase}) : super(Empty());

  @override
  Stream<RecentImportsState> mapEventToState(
    RecentImportsEvent event,
  ) async* {
    if (event is FindRecents) {
      prevQueryRange = event.range;
      yield* _runQuery(event.range);
    } else if (event is RefreshResults) {
      yield* _runQuery(prevQueryRange);
    }
  }

  Stream<RecentImportsState> _runQuery(RecentTimeRange range) async* {
    yield Loading();
    final since = range.asDate.map((v) => v.toUtc());
    final result = await usecase(Params(since: since));
    yield result.mapOrElse(
      (results) {
        // Sort the results by filename for consistency, that way images taken
        // around the same time will be near each other in the list, which is
        // helpful when applying captions.
        results.results.sort(compareRecents);
        return Loaded(results: results, range: range);
      },
      (failure) => Error(message: failure.toString()),
    );
  }
}

int compareRecents(SearchResult a, SearchResult b) {
  return a.filename.compareTo(b.filename);
}
