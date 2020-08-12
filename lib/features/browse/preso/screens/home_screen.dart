//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:tanuki/features/browse/preso/widgets/all_locations.dart';
import 'package:tanuki/features/browse/preso/widgets/all_tags.dart';
import 'package:tanuki/features/browse/preso/widgets/all_years.dart';
import 'package:tanuki/features/browse/preso/widgets/asset_count.dart';
import 'package:tanuki/features/browse/preso/widgets/assets_list.dart';

class HomeScreen extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('TANUKI'),
      ),
      body: Column(
        children: [
          AssetCount(),
          AllTags(),
          AllLocations(),
          AllYears(),
          Expanded(child: AssetsList()),
        ],
      ),
    );
  }
}
