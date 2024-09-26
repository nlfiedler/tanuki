//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:responsive_framework/responsive_framework.dart';
import 'package:tanuki/features/import/preso/widgets/ingests_form.dart';
import '../widgets/upload_form.dart'
    if (dart.library.io) '../widgets/upload_form_desktop.dart'
    if (dart.library.html) '../widgets/upload_form_web.dart';

class UploadScreen extends StatelessWidget {
  const UploadScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: ResponsiveValue(
          context,
          defaultValue: const Text('all your assets will belong to us'),
          conditionalValues: const [
            Condition.smallerThan(
              name: TABLET,
              value: Text('your assets will be ours'),
            )
          ],
        ).value,
        actions: [
          TextButton(
            onPressed: () {
              // replace the route for viewing the asset
              Navigator.pushReplacementNamed(context, '/recents');
            },
            child: const Icon(Icons.history),
          ),
        ],
      ),
      body: Column(
        children: [
          ResponsiveVisibility(
            hiddenConditions: const [
              Condition.smallerThan(name: TABLET, value: false),
            ],
            child: Padding(
              padding: const EdgeInsets.all(16.0),
              child: IngestsForm(),
            ),
          ),
          Expanded(child: UploadForm()),
        ],
      ),
    );
  }
}
