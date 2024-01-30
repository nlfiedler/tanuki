//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/features/browse/preso/bloc/all_years_bloc.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_browser_bloc.dart'
    as abb;

// ignore: use_key_in_widget_constructors
class DatesSelector extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return BlocProvider.value(
      value: BlocProvider.of<AllYearsBloc>(context),
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
            return Text('Error: ${state.message}');
          }
          if (state is Loaded) {
            final List<Year> years = List.from(state.years);
            // sort in reverse chronological order for selection convenience
            // (most recent years near the top of the dropdown menu)
            years.sort((a, b) => b.value.compareTo(a.value));
            // inject the current year if not already present so that the season
            // selection has something to select when year is unset
            final currentYear = DateTime.now().year;
            if (years[0].value != currentYear) {
              years.insert(0, Year(label: currentYear.toString(), count: 0));
            }
            return DateRangeSelectorForm(years: years);
          }
          return const Center(child: CircularProgressIndicator());
        },
      ),
    );
  }
}

class DateRangeSelectorForm extends StatefulWidget {
  final List<Year> years;

  const DateRangeSelectorForm({
    Key? key,
    required this.years,
  }) : super(key: key);

  @override
  State<DateRangeSelectorForm> createState() {
    return _DateRangeSelectorFormState();
  }
}

class _DateRangeSelectorFormState extends State<DateRangeSelectorForm> {
  final GlobalKey<FormBuilderState> _fbKey = GlobalKey<FormBuilderState>();

  Year? getSelectedYear(abb.Loaded state) {
    if (state.selectedYear != null) {
      final index = widget.years.indexWhere(
        (y) => y.value == state.selectedYear,
      );
      if (index >= 0) {
        return widget.years[index];
      }
    }
    return null;
  }

  @override
  Widget build(BuildContext context) {
    return BlocProvider.value(
      value: BlocProvider.of<abb.AssetBrowserBloc>(context),
      child: BlocConsumer<abb.AssetBrowserBloc, abb.AssetBrowserState>(
        listener: (context, state) {
          if (state is abb.Loaded) {
            // Consider if the chosen year was implicitly changed by the bloc,
            // as will happen if the season is selected before the year.
            final selectedYear = getSelectedYear(state);
            if (selectedYear != null &&
                _fbKey.currentState?.fields['year']?.value != selectedYear) {
              _fbKey.currentState?.fields['year']?.didChange(selectedYear);
            }
          }
        },
        builder: (context, state) {
          return FormBuilder(
            key: _fbKey,
            child: Row(
              children: [
                Expanded(
                  child: FormBuilderDropdown<Year>(
                    name: 'year',
                    decoration: const InputDecoration(
                      labelText: 'Year',
                      hintText: 'Any',
                    ),
                    items: [
                      const DropdownMenuItem(
                        value: null,
                        child: Text('Any'),
                      ),
                      ...widget.years
                          .map((year) => DropdownMenuItem(
                                value: year,
                                child: Text(year.label),
                              ))
                          .toList()
                    ],
                    onChanged: (Year? val) {
                      BlocProvider.of<abb.AssetBrowserBloc>(context)
                          .add(abb.SelectYear(year: val?.value));
                    },
                  ),
                ),
                const SizedBox(width: 16.0),
                Expanded(
                  child: FormBuilderDropdown<abb.Season>(
                    name: 'season',
                    decoration: const InputDecoration(
                      labelText: 'Season',
                      hintText: 'Any',
                    ),
                    items: const [
                      DropdownMenuItem(
                        value: null,
                        child: Text('Any'),
                      ),
                      DropdownMenuItem(
                        value: abb.Season.spring,
                        child: Text('Jan-Mar'),
                      ),
                      DropdownMenuItem(
                        value: abb.Season.summer,
                        child: Text('Apr-Jun'),
                      ),
                      DropdownMenuItem(
                        value: abb.Season.autumn,
                        child: Text('Jul-Sep'),
                      ),
                      DropdownMenuItem(
                        value: abb.Season.winter,
                        child: Text('Oct-Dec'),
                      ),
                    ],
                    onChanged: (abb.Season? val) {
                      BlocProvider.of<abb.AssetBrowserBloc>(context)
                          .add(abb.SelectSeason(season: val));
                    },
                  ),
                ),
              ],
            ),
          );
        },
      ),
    );
  }
}
