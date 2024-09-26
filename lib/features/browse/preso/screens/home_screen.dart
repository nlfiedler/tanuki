//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:responsive_framework/responsive_framework.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_count_bloc.dart';
import 'package:tanuki/features/browse/preso/widgets/asset_browser.dart';

// ignore: use_key_in_widget_constructors
class HomeScreen extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        leading: Image.asset('public/images/tanuki.png'),
        title: ResponsiveValue(
          context,
          defaultValue: const Text('we have your assets'),
          conditionalValues: [
            const Condition.smallerThan(
              name: MOBILE,
              value: Text('your assets'),
            ),
            const Condition.largerThan(
              name: TABLET,
              value: Text('all your assets are belong to us'),
            )
          ],
        ).value,
        actions: [
          TextButton(
            onPressed: () {
              Navigator.pushNamed(context, '/upload');
            },
            child: const Icon(Icons.file_upload),
          ),
          TextButton(
            onPressed: () {
              Navigator.pushNamed(context, '/recents');
            },
            child: const Icon(Icons.history),
          ),
        ],
      ),
      body: HomeMainWidget(),
    );
  }
}

// Show either a help screen when the database is empty, or show the asset
// browser if there is anything to see.
// ignore: use_key_in_widget_constructors
class HomeMainWidget extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return BlocProvider.value(
      value: BlocProvider.of<AssetCountBloc>(context),
      child: BlocBuilder<AssetCountBloc, AssetCountState>(
        buildWhen: (previous, current) {
          return !(previous is Loaded && current is Loading);
        },
        builder: (context, state) {
          if (state is Empty) {
            // kick off the initial remote request
            BlocProvider.of<AssetCountBloc>(context).add(LoadAssetCount());
          }
          if (state is Error) {
            return Text('Error: ${state.message}');
          }
          if (state is Loaded) {
            if (state.count == 0) {
              return buildEmptyHelp(context);
            }
            return const AssetBrowser();
          }
          return const Center(child: CircularProgressIndicator());
        },
      ),
    );
  }
}

Widget buildEmptyHelp(BuildContext context) {
  return Center(
    child: Column(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        const Text('Nothing to show, please add something.'),
        const SizedBox(height: 8.0),
        ElevatedButton(
          onPressed: () {
            Navigator.pushNamed(context, '/upload');
          },
          child: const Text('Upload'),
        ),
      ],
    ),
  );
}
