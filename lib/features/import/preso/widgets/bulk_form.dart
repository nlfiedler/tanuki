//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:intl/intl.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/input.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/core/preso/widgets/asset_display.dart';
import 'package:tanuki/features/import/preso/bloc/recent_imports_bloc.dart';

import 'bulk_submit.dart';

class BulkForm extends StatefulWidget {
  @override
  _BulkFormState createState() => _BulkFormState();
}

class _BulkFormState extends State<BulkForm> {
  final GlobalKey<FormBuilderState> _fbKey = GlobalKey<FormBuilderState>();
  // The inputValues map holds the input from the user, since the text field
  // that does not have focus will toss its contents when scrolled out of view.
  // And apparently the form builder state also loses the values?
  final Map<String, dynamic> inputValues = {};

  // Convert the input fields into a list of asset inputs.
  //
  // Nearly all of the fields will be left as None, with only the caption set to
  // whatever the user provided. All empty rows will be excluded from the final
  // result.
  List<AssetInputId> _onSubmit(
    BuildContext context,
    List<SearchResult> results,
  ) {
    // We actually don't care about the form state, since it gets wrecked when
    // the text field is discarded when it scrolls out of view, but to keep up
    // appearances we will validate the form anyway.
    if (_fbKey.currentState.saveAndValidate()) {
      final List<AssetInputId> inputs = List.generate(results.length, (idx) {
        // Undefined values are treated as empty string, so we can treat empty
        // and undefined in the same manner down below with Option.some().
        final String caption = inputValues['caption-$idx'] ?? '';
        return AssetInputId(
          id: results[idx].id,
          input: AssetInput(
            tags: [],
            caption: Option.some(caption.isEmpty ? null : caption),
            location: None(),
            datetime: None(),
            mimetype: None(),
            filename: None(),
          ),
        );
      });
      inputs.retainWhere((e) => e.input.caption is Some);
      return inputs;
    }
    return [];
  }

  @override
  Widget build(BuildContext context) {
    return BlocProvider.value(
      value: BlocProvider.of<RecentImportsBloc>(context),
      child: BlocBuilder<RecentImportsBloc, RecentImportsState>(
        builder: (context, state) {
          if (state is Loaded) {
            if (state.results.count > 0) {
              return _buildForm(state.results);
            } else {
              return Center(
                child: Text(
                  'Use the time period selectors to find pending assets.',
                ),
              );
            }
          }
          return Container();
        },
      ),
    );
  }

  Widget _buildForm(QueryResults allResults) {
    final end = allResults.count > 100 ? 100 : allResults.count;
    final results = allResults.results.sublist(0, end);
    return FormBuilder(
      key: _fbKey,
      child: Column(
        children: [
          BulkSubmit(
            onSubmit:
                results.isEmpty ? null : () => _onSubmit(context, results),
            onComplete: () {
              BlocProvider.of<RecentImportsBloc>(context).add(
                RefreshResults(),
              );
            },
          ),
          Expanded(
            child: ListView.builder(
              itemBuilder: (BuildContext context, int index) {
                final String key = 'caption-$index';
                return BulkFormRow(
                  result: results[index],
                  attribute: key,
                  initial: inputValues[key] ?? '',
                  onChanged: (val) {
                    setState(() {
                      inputValues[key] = val;
                    });
                  },
                );
              },
              itemCount: results.length,
            ),
          ),
        ],
      ),
    );
  }
}

class BulkFormRow extends StatelessWidget {
  final SearchResult result;
  final String attribute;
  final String initial;
  final ValueChanged onChanged;

  BulkFormRow({
    Key key,
    @required this.result,
    @required this.attribute,
    @required this.initial,
    @required this.onChanged,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final smallerStyle = DefaultTextStyle.of(context).style.apply(
          fontSizeFactor: 0.9,
        );
    return Row(
      children: [
        Expanded(flex: 1, child: BulkThumbnail(result: result)),
        Expanded(
          flex: 2,
          child: Padding(
            padding: const EdgeInsets.fromLTRB(0, 0, 32, 0),
            child: Column(
              children: [
                FormBuilderTextField(
                  attribute: attribute,
                  initialValue: initial,
                  decoration: InputDecoration(
                    icon: Icon(Icons.format_quote),
                    labelText: 'Caption',
                  ),
                  onChanged: onChanged,
                ),
                Padding(
                  padding: const EdgeInsets.all(8.0),
                  child: Text(
                    'Enter a description, including #tags and @location'
                    ' or @"some location"',
                    style: smallerStyle,
                  ),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }
}

class BulkThumbnail extends StatelessWidget {
  final SearchResult result;

  BulkThumbnail({Key key, @required this.result}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final datefmt = DateFormat.EEEE().add_yMMMMd();
    final dateString = datefmt.format(result.datetime);
    return Padding(
      padding: const EdgeInsets.all(8.0),
      child: SizedBox(
        width: 300.0,
        // try keeping the text in a column, the text will automatically
        // wrap to fix the available space
        child: Column(children: [
          AssetDisplay(
            assetId: result.id,
            mimetype: result.mimetype,
            displayWidth: 300,
          ),
          Text(dateString),
          Text(result.filename),
        ]),
      ),
    );
  }
}
