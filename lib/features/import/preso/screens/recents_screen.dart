//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:tanuki/container.dart';
import 'package:tanuki/features/import/preso/bloc/recent_imports_bloc.dart';
import 'package:tanuki/features/import/preso/widgets/bulk_form.dart';

class RecentsScreen extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('all your assets now belong to us'),
      ),
      body: BlocProvider<RecentImportsBloc>(
        create: (_) => getIt<RecentImportsBloc>(),
        child: BlocBuilder<RecentImportsBloc, RecentImportsState>(
          builder: (context, state) {
            if (state is Empty) {
              // kick off the initial remote request
              BlocProvider.of<RecentImportsBloc>(context).add(
                FindRecents(range: RecentTimeRange.day),
              );
            } else if (state is Loaded) {
              return Column(
                children: [RecentsSelector(), Expanded(child: BulkForm())],
              );
            } else if (state is Error) {
              return Text('Query error: ' + state.message);
            }
            return Center(child: CircularProgressIndicator());
          },
        ),
      ),
    );
  }
}

class RecentsSelector extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return BlocProvider.value(
      value: BlocProvider.of<RecentImportsBloc>(context),
      child: BlocBuilder<RecentImportsBloc, RecentImportsState>(
        builder: (context, state) {
          if (state is Loaded) {
            final biggerStyle = DefaultTextStyle.of(context).style.apply(
                  fontSizeFactor: 1.2,
                );
            return Padding(
              padding: const EdgeInsets.all(8.0),
              child: Row(
                children: [
                  Expanded(
                    child: Center(
                      child: Text(
                        'Pending assets: ${state.results.count}',
                        style: biggerStyle,
                      ),
                    ),
                  ),
                  makeButton(context, state.range, RecentTimeRange.day),
                  makeButton(context, state.range, RecentTimeRange.week),
                  makeButton(context, state.range, RecentTimeRange.month),
                  makeButton(context, state.range, RecentTimeRange.ever),
                ],
              ),
            );
          }
          return Container();
        },
      ),
    );
  }

  Widget makeButton(
    BuildContext context,
    RecentTimeRange showing,
    RecentTimeRange self,
  ) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(4, 0, 4, 0),
      child: FilterChip(
        label: Text(self.label),
        selected: showing == self,
        onSelected: (bool value) {
          if (value) {
            BlocProvider.of<RecentImportsBloc>(context).add(
              FindRecents(range: self),
            );
          }
        },
      ),
    );
  }
}

extension RecentTimeRangeExt on RecentTimeRange {
  String get label {
    switch (this) {
      case RecentTimeRange.day:
        return "Day";
      case RecentTimeRange.week:
        return "Week";
      case RecentTimeRange.month:
        return "Month";
      case RecentTimeRange.ever:
        return "All Time";
    }
    return "All Time";
  }
}
