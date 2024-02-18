//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_typeahead/flutter_typeahead.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/features/import/preso/bloc/assign_attributes_bloc.dart';
import 'package:tanuki/features/import/preso/bloc/raw_locations_bloc.dart';

class LocationSelector extends StatelessWidget {
  const LocationSelector({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BlocProvider.value(
      value: BlocProvider.of<RawLocationsBloc>(context),
      child: BlocBuilder<RawLocationsBloc, RawLocationsState>(
        buildWhen: (previous, current) {
          return !(previous is Loaded && current is Loading);
        },
        builder: (context, state) {
          if (state is Empty) {
            // kick off the initial remote request
            BlocProvider.of<RawLocationsBloc>(context).add(LoadRawLocations());
          }
          if (state is Error) {
            return Text('Error: ${state.message}');
          }
          if (state is Loaded) {
            final List<(String, AssetLocation)> prepped =
                List.from(state.locations.map((e) {
              return (e.description().toLowerCase(), e);
            }));
            return LocationSelectorStateful(locations: prepped);
          }
          return const Center(child: CircularProgressIndicator());
        },
      ),
    );
  }
}

class LocationSelectorStateful extends StatefulWidget {
  final List<(String, AssetLocation)> locations;

  const LocationSelectorStateful({
    Key? key,
    required this.locations,
  }) : super(key: key);

  @override
  State<LocationSelectorStateful> createState() =>
      _LocationSelectorStatefulState();
}

class _LocationSelectorStatefulState extends State<LocationSelectorStateful> {
  final TextEditingController _typeAheadController = TextEditingController();

  @override
  Widget build(BuildContext context) {
    return TypeAheadField<AssetLocation>(
      textFieldConfiguration: TextFieldConfiguration(
        controller: _typeAheadController,
        decoration: const InputDecoration(
          border: UnderlineInputBorder(),
          labelText: 'Select Location',
        ),
        onChanged: (value) {
          BlocProvider.of<AssignAttributesBloc>(context).add(
            AssignLocation(location: null),
          );
        },
      ),
      suggestionsCallback: (query) {
        // search on the location's lowercased description to find the most
        // suitable suggestions
        if (query.isNotEmpty) {
          // Looks complicated but this code is sorting the results by
          // the offset from the start where the query is found.
          var lowercaseQuery = query.toLowerCase();
          final results = widget.locations.where((locoreco) {
            return locoreco.$1.contains(lowercaseQuery);
          }).toList(growable: false)
            ..sort((a, b) => a.$1
                .indexOf(lowercaseQuery)
                .compareTo(b.$1.indexOf(lowercaseQuery)));
          return groomLocations(results, query);
        } else {
          return const <AssetLocation>[];
        }
      },
      itemBuilder: (context, suggestion) {
        return ListTile(
          leading: const Icon(Icons.location_on),
          title: Text(suggestion.description()),
        );
      },
      onSuggestionSelected: (suggestion) {
        _typeAheadController.text = suggestion.description();
        BlocProvider.of<AssignAttributesBloc>(context).add(
          AssignLocation(location: suggestion),
        );
      },
    );
  }
}

List<AssetLocation> groomLocations(
  List<(String, AssetLocation)> locations,
  String query,
) {
  // Need to convert from the model type to the entity type otherwise
  // when we optionally add the query itself to the list, an error occurs.
  final results = List.of(
    locations.map(
      (e) => AssetLocation(
          label: e.$2.label, city: e.$2.city, region: e.$2.region),
    ),
  );
  // Optionally add the query itself if it does not appear in the list.
  final queryExists = locations.any((e) => e.$1 == query);
  if (!queryExists) {
    results.insert(
        0,
        AssetLocation(
          label: Some(query),
          city: const None(),
          region: const None(),
        ));
  }
  return results;
}
