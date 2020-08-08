//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:tanuki/container.dart';
import 'package:tanuki/features/browse/preso/bloc/all_years_bloc.dart';

class AllYears extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return BlocProvider<AllYearsBloc>(
      create: (_) => getIt<AllYearsBloc>(),
      child: BlocBuilder<AllYearsBloc, AllYearsState>(
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
              (y) => FilterChip(
                label: Text(y.label),
                onSelected: (bool value) {},
              ),
            ));
            return Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: <Widget>[
                Wrap(
                  children: chips,
                ),
              ],
            );
          }
          return CircularProgressIndicator();
        },
      ),
    );
  }
}
