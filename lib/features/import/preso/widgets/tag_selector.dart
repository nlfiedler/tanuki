//
// Copyright (c) 2023 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_tagging_plus/flutter_tagging_plus.dart';
import 'package:tanuki/features/browse/preso/bloc/all_tags_bloc.dart';
import 'package:tanuki/features/import/preso/bloc/assign_attributes_bloc.dart';

class TagSelector extends StatelessWidget {
  const TagSelector({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BlocProvider.value(
      value: BlocProvider.of<AllTagsBloc>(context),
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
            return Text('Error: ${state.message}');
          }
          if (state is Loaded) {
            final tags = List.of(state.tags.map(
              (e) => AssetTag(label: e.label, count: e.count),
            ));
            return TagSelectorStateful(tags: tags);
          }
          return const Center(child: CircularProgressIndicator());
        },
      ),
    );
  }
}

class TagSelectorStateful extends StatefulWidget {
  final List<AssetTag> tags;

  const TagSelectorStateful({
    Key? key,
    required this.tags,
  }) : super(key: key);

  @override
  State<TagSelectorStateful> createState() => _TagSelectorState();
}

class _TagSelectorState extends State<TagSelectorStateful> {
  late List<AssetTag> _selectedTags;

  @override
  void initState() {
    _selectedTags = [];
    super.initState();
  }

  @override
  void dispose() {
    _selectedTags.clear();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return FlutterTagging<AssetTag>(
      initialItems: _selectedTags,
      textFieldConfiguration: const TextFieldConfiguration(
        decoration: InputDecoration(
          border: UnderlineInputBorder(),
          labelText: 'Select Tags',
        ),
      ),
      findSuggestions: (String query) {
        if (query.isNotEmpty) {
          // Looks complicated but this code is sorting the results by
          // the offset from the start where the query is found.
          var lowercaseQuery = query.toLowerCase();
          return widget.tags.where((tag) {
            return tag.label.toLowerCase().contains(query.toLowerCase());
          }).toList()
            ..sort((a, b) => a.label
                .toLowerCase()
                .indexOf(lowercaseQuery)
                .compareTo(b.label.toLowerCase().indexOf(lowercaseQuery)));
        } else {
          return <AssetTag>[];
        }
      },
      additionCallback: (value) {
        return AssetTag(label: value, count: 0);
      },
      configureSuggestion: (tag) {
        return SuggestionConfiguration(
          title: Text(tag.label),
          subtitle: Text(tag.count.toString()),
          additionWidget: const Chip(
            avatar: Icon(Icons.add_circle),
            label: Text('Add New Tag'),
          ),
        );
      },
      configureChip: (tag) {
        return ChipConfiguration(label: Text(tag.label));
      },
      onChanged: () {
        final tags = List.of(_selectedTags.map((e) => e.label));
        BlocProvider.of<AssignAttributesBloc>(context).add(
          AssignTags(tags: tags),
        );
      },
    );
  }
}

class AssetTag extends Taggable {
  final String label;
  final int count;

  const AssetTag({
    required this.label,
    required this.count,
  });

  @override
  List<Object> get props => [label];
}
