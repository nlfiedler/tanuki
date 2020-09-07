//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:tanuki/features/import/preso/widgets/ingests_form.dart';
import 'package:tanuki/features/import/preso/widgets/upload_form.dart';

class UploadScreen extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('all your assets will belong to us'),
      ),
      body: Column(
        children: [
          IngestsForm(),
          Expanded(child: UploadForm()),
        ],
      ),
    );
  }
}
