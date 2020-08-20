//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:tanuki/container.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/features/browse/preso/bloc/all_years_bloc.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_browser_bloc.dart'
    as abb;

class AllYears extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return BlocProvider<AllYearsBloc>(
      create: (_) => getIt<AllYearsBloc>(),
      child: BlocBuilder<AllYearsBloc, AllYearsState>(
        buildWhen: (previous, current) {
          return !(previous is Loaded && current is Loading);
        },
        builder: (context, state) {
          if (state is Empty) {
            // kick off the initial remote request
            BlocProvider.of<AllYearsBloc>(context).add(LoadAllYears());
          }
          if (state is Error) {
            return Text('Error: ' + state.message);
          }
          if (state is Loaded) {
            final List<Widget> chips = List.from(state.years.map(
              (y) => YearChip(year: y),
            ));
            return Wrap(children: chips);
          }
          return CircularProgressIndicator();
        },
      ),
    );
  }
}

class YearChip extends StatelessWidget {
  final Year year;

  const YearChip({
    Key key,
    @required this.year,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BlocProvider.value(
      value: BlocProvider.of<abb.AssetBrowserBloc>(context),
      child: BlocBuilder<abb.AssetBrowserBloc, abb.AssetBrowserState>(
        builder: (context, state) {
          bool selected = false;
          if (state is abb.Loaded) {
            selected = state.selectedYear.mapOr(
              (value) => value.toString() == year.label,
              false,
            );
          }
          return FilterChip(
            label: Text(year.label),
            selected: selected,
            onSelected: (bool value) {
              BlocProvider.of<abb.AssetBrowserBloc>(context)
                  .add(abb.ToggleYear(year: int.parse(year.label)));
            },
          );
        },
      ),
    );
  }
}
