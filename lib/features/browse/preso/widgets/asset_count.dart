//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:tanuki/container.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_count_bloc.dart';

class AssetCount extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return BlocProvider<AssetCountBloc>(
      create: (_) => getIt<AssetCountBloc>(),
      child: BlocBuilder<AssetCountBloc, AssetCountState>(
        builder: (context, state) {
          if (state is Empty) {
            // kick off the initial remote request
            BlocProvider.of<AssetCountBloc>(context).add(LoadAssetCount());
          }
          if (state is Error) {
            return Text('Error: ' + state.message);
          }
          if (state is Loaded) {
            return Text('There are ${state.count} assets');
          }
          return CircularProgressIndicator();
        },
      ),
    );
  }
}
