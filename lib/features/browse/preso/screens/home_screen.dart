//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:tanuki/features/browse/preso/widgets/all_locations.dart';
import 'package:tanuki/features/browse/preso/widgets/all_years.dart';
import 'package:tanuki/features/browse/preso/widgets/asset_count.dart';

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
          AllLocations(),
          AllYears(),
        ],
      ),
    );
  }
}
