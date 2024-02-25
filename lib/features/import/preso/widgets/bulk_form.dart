//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:intl/intl.dart';
import 'package:oxidized/oxidized.dart';
import 'package:responsive_framework/responsive_framework.dart';
import 'package:tanuki/core/domain/entities/input.dart';
import 'package:tanuki/core/domain/entities/search.dart';
import 'package:tanuki/core/preso/widgets/asset_display.dart';
import 'package:tanuki/features/browse/preso/bloc/all_tags_bloc.dart' as atb;
import 'package:tanuki/features/import/preso/bloc/assign_attributes_bloc.dart';
import 'package:tanuki/features/import/preso/bloc/providers.dart';
import 'package:tanuki/features/import/preso/bloc/raw_locations_bloc.dart'
    as rlb;
import 'package:tanuki/features/import/preso/bloc/recent_imports_bloc.dart';

import 'bulk_submit.dart';
import 'location_selector.dart';
import 'tag_selector.dart';

class BulkForm extends ConsumerWidget {
  const BulkForm({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return BlocProvider.value(
      value: BlocProvider.of<RecentImportsBloc>(context),
      child: BlocBuilder<RecentImportsBloc, RecentImportsState>(
        builder: (context, state) {
          if (state is Loaded) {
            if (state.results.count > 0) {
              return _buildForm(context, ref, state.results);
            } else {
              return const Center(
                child: Text(
                  'Use the time period selectors to find pending assets.',
                ),
              );
            }
          }
          return Container();
        },
      ),
    );
  }

  Widget _buildForm(
    BuildContext context,
    WidgetRef ref,
    QueryResults allResults,
  ) {
    final results = allResults.results;
    return BlocProvider<AssignAttributesBloc>(
      create: (_) => ref.read(assignAttributesBlocProvider),
      child: BlocBuilder<AssignAttributesBloc, AssignAttributesState>(
        builder: (context, state) {
          return Column(
            children: [
              IntrinsicHeight(
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    const Expanded(
                      flex: 2,
                      child: Padding(
                        padding: EdgeInsets.all(8.0),
                        child: TagSelector(),
                      ),
                    ),
                    const Expanded(
                      flex: 2,
                      child: Padding(
                        padding: EdgeInsets.all(8.0),
                        child: LocationSelector(),
                      ),
                    ),
                    Padding(
                      padding: const EdgeInsets.all(8.0),
                      child: Center(
                        child: BulkSubmit(
                          enabled: state.submittable,
                          onSubmit: () => List.of(state.assets.map((id) {
                            return AssetInputId(
                              id: id,
                              input: AssetInput(
                                tags: state.tags,
                                location: Option.from(state.location),
                                caption: const None(),
                                datetime: const None(),
                                filename: const None(),
                                mediaType: const None(),
                              ),
                            );
                          })),
                          onComplete: () {
                            BlocProvider.of<rlb.RawLocationsBloc>(context)
                                .add(rlb.LoadRawLocations());
                            BlocProvider.of<atb.AllTagsBloc>(context)
                                .add(atb.LoadAllTags());
                            BlocProvider.of<RecentImportsBloc>(context).add(
                              RefreshResults(),
                            );
                          },
                        ),
                      ),
                    ),
                  ],
                ),
              ),
              Expanded(child: buildThumbnails(context, results, state)),
            ],
          );
        },
      ),
    );
  }
}

Widget buildThumbnails(
  BuildContext context,
  List<SearchResult> results,
  AssignAttributesState state,
) {
  final datefmt = DateFormat.EEEE().add_yMMMMd();
  final elements = List<Widget>.from(
    results.map((e) {
      final selected = state.assets.contains(e.id);
      final dateString = datefmt.format(e.datetime.toLocal());
      return Padding(
        padding: const EdgeInsets.all(8.0),
        child: SizedBox(
          width: 300.0,
          // try keeping the text in a column, the text will automatically
          // wrap to fit the available space
          child: Column(children: [
            Stack(
              children: <Widget>[
                TextButton(
                  onPressed: () {
                    Navigator.pushNamed(context, '/edit', arguments: e.id);
                  },
                  child: AssetDisplay(
                    assetId: e.id,
                    mediaType: e.mediaType,
                    displayWidth: 300,
                  ),
                ),
                Positioned(
                  top: 8,
                  right: 8,
                  child: TextButton(
                    onPressed: () {
                      BlocProvider.of<AssignAttributesBloc>(context).add(
                        ToggleAsset(assetId: e.id),
                      );
                    },
                    child: Icon(selected
                        ? Icons.check_circle
                        : Icons.add_circle_outline),
                  ),
                ),
              ],
            ),
            Text(dateString),
            ResponsiveVisibility(
              hiddenConditions: [
                Condition.smallerThan(name: TABLET, value: false),
              ],
              child: Text(
                e.location.map((e) => e.description()).unwrapOr(e.filename),
              ),
            ),
          ]),
        ),
      );
    }),
  );
  return SingleChildScrollView(
    child: Wrap(children: elements),
  );
}
