//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:intl/intl.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/environment_config.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_browser_bloc.dart';

class AssetsList extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return BlocProvider.value(
      value: BlocProvider.of<AssetBrowserBloc>(context),
      child: BlocBuilder<AssetBrowserBloc, AssetBrowserState>(
        builder: (context, state) {
          if (state is Error) {
            return Text('Error: ' + state.message);
          }
          if (state is Loaded) {
            return buildThumbnails(context, state.results.results);
          }
          return Center(child: CircularProgressIndicator());
        },
      ),
    );
  }
}

const thumbnail300 = '/api/thumbnail/300/300/';

Widget buildThumbnails(BuildContext context, List<SearchResult> results) {
  final datefmt = DateFormat.EEEE().add_yMMMMd();
  final elements = List<Widget>.from(
    results.map((e) {
      final uri = '${EnvironmentConfig.base_url}$thumbnail300${e.id}';
      final dateString = datefmt.format(e.datetime.toLocal());
      return Padding(
        padding: const EdgeInsets.all(8.0),
        child: FlatButton(
          onPressed: () {
            Navigator.pushNamed(context, '/asset', arguments: e.id);
          },
          child: Padding(
            padding: const EdgeInsets.all(8.0),
            child: Column(
              children: [
                Image.network(uri, fit: BoxFit.contain),
                SizedBox(
                  width: 300.0,
                  // try keeping the text in a column, the text will automatically
                  // wrap to fix the available space
                  child: Column(children: [
                    Text(dateString),
                    Text(e.filename),
                  ]),
                ),
              ],
            ),
          ),
        ),
      );
    }),
  );
  return SingleChildScrollView(
    child: Wrap(children: elements),
  );
}
