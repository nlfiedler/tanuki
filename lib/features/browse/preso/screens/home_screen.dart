//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:tanuki/features/browse/preso/widgets/asset_browser.dart';

class HomeScreen extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('all your assets are belong to us'),
        actions: [
          FlatButton(
            child: Icon(Icons.file_upload),
            onPressed: () {
              Navigator.pushNamed(context, '/upload');
            },
          ),
          FlatButton(
            child: Icon(Icons.history),
            onPressed: () {
              Navigator.pushNamed(context, '/recents');
            },
          ),
        ],
      ),
      body: AssetBrowser(),
    );
  }
}
