//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_chips_input/flutter_chips_input.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/features/browse/preso/bloc/all_locations_bloc.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_browser_bloc.dart'
    as abb;

class LocationsSelector extends StatelessWidget {
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
            return Text('Error: ' + state.message);
          }
          if (state is Loaded) {
            return LocationSelectorForm(locations: state.locations);
          }
          return Center(child: CircularProgressIndicator());
        },
      ),
    );
  }
}

class LocationSelectorForm extends StatefulWidget {
  final List<Location> locations;

  const LocationSelectorForm({
    Key key,
    @required this.locations,
  }) : super(key: key);

  @override
  _LocationSelectorFormState createState() => _LocationSelectorFormState();
}

class _LocationSelectorFormState extends State<LocationSelectorForm> {
  final GlobalKey<FormBuilderState> _fbKey = GlobalKey<FormBuilderState>();
  final GlobalKey<ChipsInputState> _chipKey = GlobalKey();

  @override
  Widget build(BuildContext context) {
    return BlocProvider.value(
      value: BlocProvider.of<abb.AssetBrowserBloc>(context),
      child: BlocBuilder<abb.AssetBrowserBloc, abb.AssetBrowserState>(
        builder: (context, state) {
          final List<String> selected =
              state is abb.Loaded ? state.selectedLocations : [];
          return FormBuilder(
            key: _fbKey,
            child: buildAttributeSelector(context, selected),
          );
        },
      ),
    );
  }

  Widget buildAttributeSelector(BuildContext context, List<String> selected) {
    return Stack(
      children: [
        buildChipsInput(context),
        Align(
          alignment: Alignment.centerRight,
          child: DropdownButton(
            onChanged: (value) {
              // Toggle the item in the selected list.
              final values = toggleSelection(selected, value.label, value);
              BlocProvider.of<abb.AssetBrowserBloc>(context)
                  .add(abb.SelectLocations(locations: values));
            },
            items: [
              for (final location in widget.locations)
                DropdownMenuItem(
                  value: location,
                  child: Text(location.label),
                ),
            ],
          ),
        ),
      ],
    );
  }

  ChipsInput buildChipsInput(BuildContext context) {
    return ChipsInput(
      key: _chipKey,
      decoration: const InputDecoration(labelText: 'Locations'),
      onChanged: (val) {
        // Need to explicitly convert the value, whatever it is, even a
        // list of strings, to a list of strings, so may as well use a
        // list of Locations throughout.
        final List<String> locations = List.from(val.map((t) => t.label));
        BlocProvider.of<abb.AssetBrowserBloc>(context)
            .add(abb.SelectLocations(locations: locations));
      },
      maxChips: 10,
      findSuggestions: (String query) {
        if (query.isNotEmpty) {
          // Looks complicated but this code is sorting the results by
          // the offset from the start where the query is found.
          var lowercaseQuery = query.toLowerCase();
          return widget.locations.where((location) {
            return location.label.toLowerCase().contains(query.toLowerCase());
          }).toList(growable: false)
            ..sort((a, b) => a.label
                .toLowerCase()
                .indexOf(lowercaseQuery)
                .compareTo(b.label.toLowerCase().indexOf(lowercaseQuery)));
        } else {
          return const <Location>[];
        }
      },
      chipBuilder: (context, state, location) {
        return InputChip(
          key: ObjectKey(location),
          label: Text(location.label),
          onDeleted: () => state.deleteChip(location),
          materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
        );
      },
      suggestionBuilder: (context, state, location) {
        return ListTile(
          key: ObjectKey(location),
          leading: Icon(Icons.label),
          title: Text(location.label),
          onTap: () => state.selectSuggestion(location),
        );
      },
    );
  }

  // Side-effect: adds/removes a chip from the chips input.
  List<String> toggleSelection(
    List<String> selected,
    String label,
    Location value,
  ) {
    final List<String> values = List.from(selected);
    if (values.contains(label)) {
      values.remove(label);
      _chipKey.currentState.deleteChip(value);
    } else {
      values.add(label);
      _chipKey.currentState.selectSuggestion(value);
    }
    return values;
  }
}
