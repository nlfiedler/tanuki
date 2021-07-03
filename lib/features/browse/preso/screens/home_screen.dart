//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_count_bloc.dart';
import 'package:tanuki/features/browse/preso/widgets/asset_browser.dart';

class HomeScreen extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('all your assets are belong to us'),
        actions: [
          TextButton(
            onPressed: () {
              Navigator.pushNamed(context, '/upload');
            },
            child: Icon(Icons.file_upload),
          ),
          TextButton(
            onPressed: () {
              Navigator.pushNamed(context, '/recents');
            },
            child: Icon(Icons.history),
          ),
        ],
      ),
      body: HomeMainWidget(),
    );
  }
}

// Show either a help screen when the database is empty, or show the asset
// browser if there is anything to see.
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
            return Text('Error: ' + state.message);
          }
          if (state is Loaded) {
            if (state.count == 0) {
              return buildEmptyHelp(context);
            }
            return AssetBrowser();
          }
          return Center(child: CircularProgressIndicator());
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
        Text('Nothing to show, please add something.'),
        SizedBox(height: 8.0),
        ElevatedButton(
          onPressed: () {
            Navigator.pushNamed(context, '/upload');
          },
          child: Text('Upload'),
        ),
      ],
    ),
  );
}
