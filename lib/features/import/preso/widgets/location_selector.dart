//
// Copyright (c) 2023 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_typeahead/flutter_typeahead.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/features/browse/preso/bloc/all_locations_bloc.dart';
import 'package:tanuki/features/import/preso/bloc/assign_attributes_bloc.dart';

class LocationSelector extends StatelessWidget {
  const LocationSelector({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BlocProvider.value(
      value: BlocProvider.of<AllLocationsBloc>(context),
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
            return Text('Error: ${state.message}');
          }
          if (state is Loaded) {
            return LocationSelectorStateful(locations: state.locations);
          }
          return const Center(child: CircularProgressIndicator());
        },
      ),
    );
  }
}

class LocationSelectorStateful extends StatefulWidget {
  final List<Location> locations;

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
    return TypeAheadField<Location>(
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
        if (query.isNotEmpty) {
          // Looks complicated but this code is sorting the results by
          // the offset from the start where the query is found.
          var lowercaseQuery = query.toLowerCase();
          final results = widget.locations.where((location) {
            return location.label.toLowerCase().contains(query.toLowerCase());
          }).toList(growable: false)
            ..sort((a, b) => a.label
                .toLowerCase()
                .indexOf(lowercaseQuery)
                .compareTo(b.label.toLowerCase().indexOf(lowercaseQuery)));
          return groomLocations(results, query.toLowerCase());
        } else {
          return const <Location>[];
        }
      },
      itemBuilder: (context, suggestion) {
        return ListTile(
          leading: const Icon(Icons.location_on),
          title: Text(suggestion.label),
        );
      },
      onSuggestionSelected: (suggestion) {
        _typeAheadController.text = suggestion.label;
        BlocProvider.of<AssignAttributesBloc>(context).add(
          AssignLocation(location: suggestion.label),
        );
      },
    );
  }
}

List<Location> groomLocations(List<Location> locations, String query) {
  // Need to convert from the model type to the entity type otherwise
  // when we optionally add the query itself to the list, an error occurs.
  final results = List.of(
    locations.map((e) => Location(label: e.label, count: e.count)),
  );
  // Optionally add the query itself if it does not appear in the list.
  final queryExists = locations.any((e) => e.label.toLowerCase() == query);
  if (!queryExists) {
    results.insert(0, Location(label: query, count: 0));
  }
  return results;
}
