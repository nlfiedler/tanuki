//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:tanuki/core/domain/entities/search.dart';
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
            return buildResultsList(context, state.results.results);
          }
          return Center(child: CircularProgressIndicator());
        },
      ),
    );
  }
}

Widget buildResultsList(BuildContext context, List<SearchResult> results) {
  final elements = List<Widget>.from(
    results.map((e) {
      return Card(
        child: ListTile(
          leading: Icon(Icons.photo),
          title: Text(e.filename),
          subtitle: Text('Location: ' + e.location.unwrapOr('(none)')),
        ),
      );
    }),
  );
  return ListView(children: elements);
}
