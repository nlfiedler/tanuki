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
      body: AssetBrowser(),
    );
  }
}
