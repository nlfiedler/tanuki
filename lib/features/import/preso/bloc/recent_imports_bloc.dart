//
// Copyright (c) 2023 Nathan Fiedler
//
import 'dart:async';
import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:equatable/equatable.dart';
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
        return Some(now.subtract(const Duration(days: 1)));
      case RecentTimeRange.week:
        return Some(now.subtract(const Duration(days: 7)));
      case RecentTimeRange.month:
        return Some(now.subtract(const Duration(days: 30)));
      case RecentTimeRange.ever:
        return const None();
    }
  }
}

class FindRecents extends RecentImportsEvent {
  final RecentTimeRange range;

  FindRecents({required this.range});
}

class ShowPage extends RecentImportsEvent {
  final int page;

  ShowPage({
    required this.page,
  });
}

class SetPageSize extends RecentImportsEvent {
  final int size;

  SetPageSize({
    required this.size,
  });
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
  final int pageSize;
  final int pageNumber;
  final int lastPage;

  Loaded({
    required this.results,
    required this.range,
    required this.pageSize,
    required this.pageNumber,
    required this.lastPage,
  });

  @override
  List<Object> get props => [range, results, pageNumber];
}

class Error extends RecentImportsState {
  final String message;

  Error({required this.message});

  @override
  List<Object> get props => [message];
}

//
// bloc
//

class RecentImportsBloc extends Bloc<RecentImportsEvent, RecentImportsState> {
  final QueryRecents usecase;
  RecentTimeRange prevQueryRange = RecentTimeRange.day;
  int pageSize = 18;
  int pageNumber = 1;

  RecentImportsBloc({required this.usecase}) : super(Empty()) {
    // enforce sequential ordering of event mapping due to the asynchronous
    // nature of this bloc
    on<RecentImportsEvent>(_onEvent, transformer: sequential());
  }

  FutureOr<void> _onEvent(
    RecentImportsEvent event,
    Emitter<RecentImportsState> emit,
  ) async {
    if (event is FindRecents) {
      if (event.range != prevQueryRange) {
        pageNumber = 1;
      }
      prevQueryRange = event.range;
      return _runQuery(event.range, emit);
    } else if (event is RefreshResults) {
      pageNumber = 1;
      return _runQuery(prevQueryRange, emit);
    } else if (event is ShowPage) {
      if (state is Loaded) {
        pageNumber = event.page;
        return _runQuery(prevQueryRange, emit);
      }
    } else if (event is SetPageSize) {
      if (state is Loaded) {
        pageSize = event.size;
        pageNumber = 1;
        return _runQuery(prevQueryRange, emit);
      }
    }
  }

  Future<void> _runQuery(
    RecentTimeRange range,
    Emitter<RecentImportsState> emit,
  ) async {
    emit(Loading());
    final since = range.asDate.map((v) => v.toUtc());
    final offset = pageSize * (pageNumber - 1);
    final result = await usecase(Params(
      since: since,
      count: Some(pageSize),
      offset: Some(offset),
    ));
    emit(result.mapOrElse(
      (results) {
        final lastPage = (results.count / pageSize).ceil();
        return Loaded(
          results: results,
          range: range,
          pageSize: pageSize,
          pageNumber: lastPage > 0 ? pageNumber : 0,
          lastPage: lastPage,
        );
      },
      (failure) => Error(message: failure.toString()),
    ));
  }
}
