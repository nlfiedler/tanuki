//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:tanuki/container.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/features/browse/preso/bloc/all_locations_bloc.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_browser_bloc.dart'
    as abb;

class AllLocations extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return BlocProvider<AllLocationsBloc>(
      create: (_) => getIt<AllLocationsBloc>(),
      child: BlocBuilder<AllLocationsBloc, AllLocationsState>(
        buildWhen: (previous, current) {
          return !(previous is Loaded && current is Loading);
        },
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
              (y) => LocationChip(location: y),
            ));
            return Wrap(children: chips);
          }
          return CircularProgressIndicator();
        },
      ),
    );
  }
}

class LocationChip extends StatelessWidget {
  final Location location;

  const LocationChip({
    Key key,
    @required this.location,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BlocProvider.value(
      value: BlocProvider.of<abb.AssetBrowserBloc>(context),
      child: BlocBuilder<abb.AssetBrowserBloc, abb.AssetBrowserState>(
        builder: (context, state) {
          bool selected = false;
          if (state is abb.Loaded) {
            if (state.selectedLocations.contains(location.label)) {
              selected = true;
            }
          }
          return FilterChip(
            label: Text(location.label),
            selected: selected,
            onSelected: (bool value) {
              BlocProvider.of<abb.AssetBrowserBloc>(context)
                  .add(abb.ToggleLocation(location: location.label));
            },
          );
        },
      ),
    );
  }
}
