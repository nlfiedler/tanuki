//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:tanuki/container.dart';
import 'package:tanuki/features/browse/preso/bloc/all_locations_bloc.dart';

class AllLocations extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return BlocProvider<AllLocationsBloc>(
      create: (_) => getIt<AllLocationsBloc>(),
      child: BlocBuilder<AllLocationsBloc, AllLocationsState>(
        builder: (context, state) {
          if (state is Empty) {
            // kick off the initial remote request
            BlocProvider.of<AllLocationsBloc>(context).add(LoadAllLocations());
          }
          if (state is Error) {
            return Text('Error: ' + state.message);
          }
          if (state is Loaded) {
            final List<Widget> chips = List.from(state.locations.map(
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
