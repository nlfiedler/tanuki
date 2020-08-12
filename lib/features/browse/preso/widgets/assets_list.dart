//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:tanuki/container.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/features/browse/preso/bloc/query_assets_bloc.dart';

class AssetsList extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return BlocProvider<QueryAssetsBloc>(
      create: (_) => getIt<QueryAssetsBloc>(),
      child: BlocBuilder<QueryAssetsBloc, QueryAssetsState>(
        builder: (context, state) {
          if (state is Empty) {
            // kick off the initial remote request
            final params = SearchParams(tags: ['cat']);
            BlocProvider.of<QueryAssetsBloc>(context).add(LoadQueryAssets(
              params: params,
              count: 10,
              offset: 0,
            ));
          }
          if (state is Error) {
            return Text('Error: ' + state.message);
          }
          if (state is Loaded) {
            return buildResultsList(context, state.results.results);
          }
          return CircularProgressIndicator();
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
