//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:tanuki/container.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_browser_bloc.dart';
import 'all_locations.dart';
import 'all_tags.dart';
import 'all_years.dart';
import 'assets_list.dart';
import 'page_controls.dart';

class AssetBrowser extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return BlocProvider<AssetBrowserBloc>(
      create: (_) => getIt<AssetBrowserBloc>(),
      child: BlocBuilder<AssetBrowserBloc, AssetBrowserState>(
        buildWhen: (previous, current) {
          return !(previous is Loaded && current is Loading);
        },
        builder: (context, state) {
          if (state is Empty) {
            // kick off the initial remote request
            BlocProvider.of<AssetBrowserBloc>(context).add(LoadInitialAssets());
          }
          if (state is Error) {
            return Text('Error: ' + state.message);
          }
          if (state is Loaded) {
            return Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                AllTags(),
                AllLocations(),
                AllYears(),
                PageControls(),
                Expanded(child: AssetsList()),
              ],
            );
          }
          return CircularProgressIndicator();
        },
      ),
    );
  }
}
