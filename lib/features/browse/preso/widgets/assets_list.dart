//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:intl/intl.dart';
import 'package:responsive_framework/responsive_framework.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/core/preso/widgets/asset_display.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_browser_bloc.dart';

class AssetsList extends StatelessWidget {
  const AssetsList({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BlocProvider.value(
      value: BlocProvider.of<AssetBrowserBloc>(context),
      child: BlocBuilder<AssetBrowserBloc, AssetBrowserState>(
        builder: (context, state) {
          if (state is Error) {
            return Text('Error: ${state.message}');
          }
          if (state is Loaded) {
            return buildThumbnails(context, state.results.results);
          }
          return const Center(child: CircularProgressIndicator());
        },
      ),
    );
  }
}

Widget buildThumbnails(BuildContext context, List<SearchResult> results) {
  final datefmt = DateFormat.EEEE().add_yMMMMd();
  final elements = List<Widget>.from(
    results.map((e) {
      final dateString = datefmt.format(e.datetime.toLocal());
      return Padding(
        padding: const EdgeInsets.all(8.0),
        child: TextButton(
          onPressed: () {
            Navigator.pushNamed(context, '/asset', arguments: e.id);
          },
          child: Padding(
            padding: const EdgeInsets.all(8.0),
            child: SizedBox(
              width: 300.0,
              // try keeping the text in a column, the text will automatically
              // wrap to fix the available space
              child: Column(children: [
                AssetDisplay(
                  assetId: e.id,
                  mimetype: e.mimetype,
                  displayWidth: 300,
                ),
                Text(dateString),
                ResponsiveVisibility(
                  hiddenConditions: [
                    Condition.smallerThan(name: TABLET, value: false),
                  ],
                  child: Text(e.filename),
                ),
              ]),
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
