//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:tanuki/container.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/features/browse/preso/bloc/all_tags_bloc.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_browser_bloc.dart'
    as abb;

class AllTags extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return BlocProvider<AllTagsBloc>(
      create: (_) => getIt<AllTagsBloc>(),
      child: BlocBuilder<AllTagsBloc, AllTagsState>(
        buildWhen: (previous, current) {
          return !(previous is Loaded && current is Loading);
        },
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
              (y) => TagChip(tag: y),
            ));
            return Wrap(children: chips);
          }
          return CircularProgressIndicator();
        },
      ),
    );
  }
}

class TagChip extends StatelessWidget {
  final Tag tag;

  const TagChip({
    Key key,
    @required this.tag,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BlocProvider.value(
      value: BlocProvider.of<abb.AssetBrowserBloc>(context),
      child: BlocBuilder<abb.AssetBrowserBloc, abb.AssetBrowserState>(
        builder: (context, state) {
          bool selected = false;
          if (state is abb.Loaded) {
            if (state.selectedTags.contains(tag.label)) {
              selected = true;
            }
          }
          return FilterChip(
            label: Text(tag.label),
            selected: selected,
            onSelected: (bool value) {
              BlocProvider.of<abb.AssetBrowserBloc>(context)
                  .add(abb.ToggleTag(tag: tag.label));
            },
          );
        },
      ),
    );
  }
}
