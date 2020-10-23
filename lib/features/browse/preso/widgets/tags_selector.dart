//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';
import 'package:tanuki/features/browse/preso/bloc/all_tags_bloc.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_browser_bloc.dart'
    as abb;

class TagsSelector extends StatelessWidget {
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
            return Text('Error: ' + state.message);
          }
          if (state is Loaded) {
            return TagSelectorForm(tags: state.tags);
          }
          return Center(child: CircularProgressIndicator());
        },
      ),
    );
  }
}

class TagSelectorForm extends StatefulWidget {
  final List<Tag> tags;

  const TagSelectorForm({
    Key key,
    @required this.tags,
  }) : super(key: key);

  @override
  _TagSelectorFormState createState() => _TagSelectorFormState();
}

class _TagSelectorFormState extends State<TagSelectorForm> {
  final GlobalKey<FormBuilderState> _fbKey = GlobalKey<FormBuilderState>();

  @override
  Widget build(BuildContext context) {
    return BlocProvider.value(
      value: BlocProvider.of<abb.AssetBrowserBloc>(context),
      child: BlocBuilder<abb.AssetBrowserBloc, abb.AssetBrowserState>(
        builder: (context, state) {
          final List<String> selected =
              state is abb.Loaded ? state.selectedTags : [];
          return FormBuilder(
            key: _fbKey,
            child: buildAttributeSelector(context, selected),
          );
        },
      ),
    );
  }

  Widget buildAttributeSelector(BuildContext context, List<String> selected) {
    return Stack(
      children: [
        buildChipsInput(context),
        Align(
          alignment: Alignment.centerRight,
          child: DropdownButton(
            onChanged: (value) {
              // Toggle the item in the selected list.
              final values = toggleSelection(selected, value.label);
              // Update the chips input form field to match.
              updateChipsInput(values);
              BlocProvider.of<abb.AssetBrowserBloc>(context)
                  .add(abb.SelectTags(tags: values));
            },
            items: [
              for (final tag in widget.tags)
                DropdownMenuItem(
                  value: tag,
                  child: Text(tag.label),
                ),
            ],
          ),
        ),
      ],
    );
  }

  FormBuilderChipsInput buildChipsInput(BuildContext context) {
    return FormBuilderChipsInput(
      attribute: 'tags',
      decoration: const InputDecoration(labelText: 'Tags'),
      onChanged: (val) {
        // Need to explicitly convert the value, whatever it is, even a
        // list of strings, to a list of strings, so may as well use a
        // list of Tags throughout.
        final List<String> tags = List.from(val.map((t) => t.label));
        BlocProvider.of<abb.AssetBrowserBloc>(context)
            .add(abb.SelectTags(tags: tags));
      },
      maxChips: 10,
      findSuggestions: (String query) {
        if (query.isNotEmpty) {
          // Looks complicated but this code is sorting the results by
          // the offset from the start where the query is found.
          var lowercaseQuery = query.toLowerCase();
          return widget.tags.where((tag) {
            return tag.label.toLowerCase().contains(query.toLowerCase());
          }).toList(growable: false)
            ..sort((a, b) => a.label
                .toLowerCase()
                .indexOf(lowercaseQuery)
                .compareTo(b.label.toLowerCase().indexOf(lowercaseQuery)));
        } else {
          return const <Tag>[];
        }
      },
      chipBuilder: (context, state, tag) {
        return InputChip(
          key: ObjectKey(tag),
          label: Text(tag.label),
          onDeleted: () => state.deleteChip(tag),
          materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
        );
      },
      suggestionBuilder: (context, state, tag) {
        return ListTile(
          key: ObjectKey(tag),
          leading: Icon(Icons.label),
          title: Text(tag.label),
          onTap: () => state.selectSuggestion(tag),
        );
      },
    );
  }

  void updateChipsInput(List<String> values) {
    // Trying to rebuild the form with a different initialValue has no effect,
    // so instead simulate a user action by telling the field that it has been
    // changed (this is, in fact, the way this is intended to work).
    final List<Tag> tags = List.of(values.map(
      (v) => Tag(label: v, count: 1),
    ));
    _fbKey.currentState.fields['tags'].currentState.didChange(tags);
  }
}

List<String> toggleSelection(List<String> selected, String value) {
  final List<String> values = List.from(selected);
  if (values.contains(value)) {
    values.remove(value);
  } else {
    values.add(value);
  }
  return values;
}
