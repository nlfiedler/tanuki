//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:intl/intl.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/features/browse/preso/bloc/all_years_bloc.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_browser_bloc.dart'
    as abb;

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
            return Text('Error: ' + state.message);
          }
          if (state is Loaded) {
            return DateRangeSelectorForm(years: state.years);
          }
          return Center(child: CircularProgressIndicator());
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
  _DateRangeSelectorFormState createState() {
    if (years.isEmpty) {
      return _DateRangeSelectorFormState(
        firstDate: DateTime.utc(1900),
        lastDate: DateTime.utc(2100),
      );
    }
    final firstYear = int.parse(years.first.label);
    final firstDate = DateTime.utc(firstYear, 1, 1);
    final lastYear = int.parse(years.last.label);
    final lastDate = DateTime.utc(lastYear, 12, 31);
    return _DateRangeSelectorFormState(
      firstDate: firstDate,
      lastDate: lastDate,
    );
  }
}

class _DateRangeSelectorFormState extends State<DateRangeSelectorForm> {
  final GlobalKey<FormBuilderState> _fbKey = GlobalKey<FormBuilderState>();
  final DateTime firstDate;
  final DateTime lastDate;

  _DateRangeSelectorFormState({
    required this.firstDate,
    required this.lastDate,
  }) : super();

  @override
  Widget build(BuildContext context) {
    // use the default first/last dates from flutter form builder unless the
    // desired dates are outside of that range
    var afterFirstDate = DateTime.utc(1900);
    var beforeLastDate = DateTime.utc(2100);
    if (firstDate.isBefore(afterFirstDate)) {
      afterFirstDate = firstDate;
    }
    if (lastDate.isAfter(beforeLastDate)) {
      beforeLastDate = lastDate;
    }
    return BlocProvider.value(
      value: BlocProvider.of<abb.AssetBrowserBloc>(context),
      child: BlocBuilder<abb.AssetBrowserBloc, abb.AssetBrowserState>(
        builder: (context, state) {
          return FormBuilder(
            key: _fbKey,
            child: Row(
              children: [
                Expanded(
                  child: FormBuilderDateTimePicker(
                    name: 'afterDate',
                    helpText: 'Select earliest date',
                    format: DateFormat.yMd(),
                    inputType: InputType.date,
                    firstDate: afterFirstDate,
                    decoration: InputDecoration(
                      labelText: 'After',
                      suffixIcon: IconButton(
                        icon: Icon(Icons.clear),
                        onPressed: () {
                          BlocProvider.of<abb.AssetBrowserBloc>(context)
                              .add(abb.SetAfterDate(date: null));
                          _fbKey.currentState?.fields['afterDate']?.reset();
                        },
                      ),
                    ),
                    onChanged: (DateTime? val) {
                      BlocProvider.of<abb.AssetBrowserBloc>(context)
                          .add(abb.SetAfterDate(date: val));
                    },
                  ),
                ),
                SizedBox(width: 8.0),
                Expanded(
                  child: FormBuilderDateTimePicker(
                    name: 'beforeDate',
                    helpText: 'Select latest date',
                    format: DateFormat.yMd(),
                    inputType: InputType.date,
                    lastDate: beforeLastDate,
                    decoration: InputDecoration(
                      labelText: 'Before',
                      suffixIcon: IconButton(
                        icon: Icon(Icons.clear),
                        onPressed: () {
                          BlocProvider.of<abb.AssetBrowserBloc>(context)
                              .add(abb.SetBeforeDate(date: null));
                          _fbKey.currentState?.fields['beforeDate']?.reset();
                        },
                      ),
                    ),
                    onChanged: (DateTime? val) {
                      BlocProvider.of<abb.AssetBrowserBloc>(context)
                          .add(abb.SetBeforeDate(date: val));
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
