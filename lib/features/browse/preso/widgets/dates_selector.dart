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
    Key key,
    @required this.years,
  }) : super(key: key);

  @override
  _DateRangeSelectorFormState createState() {
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
    @required this.firstDate,
    @required this.lastDate,
  }) : super();

  @override
  Widget build(BuildContext context) {
    final datefmt = DateFormat.yMd();
    return BlocProvider.value(
      value: BlocProvider.of<abb.AssetBrowserBloc>(context),
      child: BlocBuilder<abb.AssetBrowserBloc, abb.AssetBrowserState>(
        builder: (context, state) {
          return FormBuilder(
            key: _fbKey,
            child: FormBuilderDateRangePicker(
              attribute: 'dates',
              format: datefmt,
              firstDate: firstDate,
              lastDate: lastDate,
              decoration: const InputDecoration(labelText: 'Dates'),
              onChanged: (val) {
                // Without a form-save invocation, the value transformer does
                // not get called, so do the parsing here and send the dates
                // over to the bloc to take effect immediately.
                final dates = parseRangeValue(val, datefmt);
                BlocProvider.of<abb.AssetBrowserBloc>(context)
                    .add(abb.SelectDates(dates: dates));
              },
              valueTransformer: (text) {
                // Due to the following bug, need to work around the issue of
                // the value being a formatted date range, rather than a list of
                // DateTime (which is the case in the latest 3.x release).
                //
                // https://github.com/danvick/flutter_form_builder/issues/458
                //
                return parseRangeValue(text, datefmt);
              },
            ),
          );
        },
      ),
    );
  }
}

List<DateTime> parseRangeValue(String val, DateFormat fmt) {
  if (val.isEmpty) {
    return [];
  }
  if (val.contains(' - ')) {
    final parts = val.split(' - ');
    return List.of(parts.map((v) => fmt.parse(v)));
  }
  return [fmt.parse(val)];
}
