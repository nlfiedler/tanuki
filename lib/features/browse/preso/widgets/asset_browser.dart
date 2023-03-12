//
// Copyright (c) 2023 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:responsive_framework/responsive_framework.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_browser_bloc.dart';
import 'assets_list.dart';
import 'dates_selector.dart';
import 'locations_selector.dart';
import 'page_controls.dart';
import 'tags_selector.dart';

class AssetBrowser extends StatelessWidget {
  const AssetBrowser({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BlocProvider.value(
      value: BlocProvider.of<AssetBrowserBloc>(context),
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
            return Text('Error: ${state.message}');
          }
          if (state is Loaded) {
            return Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Row(
                  crossAxisAlignment: CrossAxisAlignment.end,
                  children: [
                    Expanded(
                      flex: 3,
                      child: Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16.0),
                        child: TagsSelector(),
                      ),
                    ),
                    Expanded(
                      flex: 2,
                      child: Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16.0),
                        child: LocationsSelector(),
                      ),
                    ),
                    ResponsiveVisibility(
                      hiddenWhen: const [
                        Condition.smallerThan(name: TABLET),
                      ],
                      child: Expanded(
                        flex: 2,
                        child: Padding(
                          padding: const EdgeInsets.symmetric(horizontal: 16.0),
                          child: DatesSelector(),
                        ),
                      ),
                    ),
                  ],
                ),
                const PageControls(),
                const Expanded(child: AssetsList()),
              ],
            );
          }
          return const Center(child: CircularProgressIndicator());
        },
      ),
    );
  }
}
