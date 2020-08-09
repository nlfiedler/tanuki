//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:tanuki/container.dart';
import 'package:tanuki/features/browse/preso/bloc/all_tags_bloc.dart';

class AllTags extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return BlocProvider<AllTagsBloc>(
      create: (_) => getIt<AllTagsBloc>(),
      child: BlocBuilder<AllTagsBloc, AllTagsState>(
        builder: (context, state) {
          if (state is Empty) {
            // kick off the initial remote request
            BlocProvider.of<AllTagsBloc>(context).add(LoadAllTags());
          }
          if (state is Error) {
            return Text('Error: ' + state.message);
          }
          if (state is Loaded) {
            final List<Widget> chips = List.from(state.tags.map(
              (y) => FilterChip(
                label: Text(y.label),
                onSelected: (bool value) {},
              ),
            ));
            return Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: <Widget>[
                Wrap(
                  children: chips,
                ),
              ],
            );
          }
          return CircularProgressIndicator();
        },
      ),
    );
  }
}
