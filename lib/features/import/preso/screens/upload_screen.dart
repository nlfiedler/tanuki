//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:tanuki/features/import/preso/widgets/ingests_form.dart';
import '../widgets/upload_form.dart'
    if (dart.library.io) '../widgets/upload_form_desktop.dart'
    if (dart.library.html) '../widgets/upload_form_web.dart';

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
