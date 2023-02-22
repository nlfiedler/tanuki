//
// Copyright (c) 2023 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tanuki/features/import/preso/bloc/ingest_assets_bloc.dart';
import 'package:tanuki/features/import/preso/bloc/providers.dart';

// ignore: use_key_in_widget_constructors
class IngestsForm extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return BlocProvider<IngestAssetsBloc>(
      create: (_) => ref.read(ingestAssetsBlocProvider),
      child: BlocBuilder<IngestAssetsBloc, IngestAssetsState>(
        builder: (context, state) {
          return Padding(
            padding: const EdgeInsets.all(8.0),
            child: Column(
              children: [
                _buildActionRow(context),
                _buildStatusRow(context, state),
              ],
            ),
          );
        },
      ),
    );
  }

  Row _buildActionRow(BuildContext context) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        RichText(
          text: TextSpan(
            text: 'To import files in the ',
            style: DefaultTextStyle.of(context).style,
            children: const <TextSpan>[
              TextSpan(
                text: 'uploads',
                style: TextStyle(
                  fontFamily: 'RobotoMono',
                  fontStyle: FontStyle.italic,
                ),
              ),
              TextSpan(text: ' directory, click on'),
            ],
          ),
        ),
        const SizedBox(width: 8.0),
        ElevatedButton(
          onPressed: () {
            BlocProvider.of<IngestAssetsBloc>(context).add(
              ProcessUploads(),
            );
          },
          child: const Text('IMPORT'),
        ),
      ],
    );
  }

  Widget _buildStatusRow(BuildContext context, IngestAssetsState state) {
    if (state is Processing) {
      return const Center(child: CircularProgressIndicator());
    }
    if (state is Error) {
      return Center(child: Text('Import error: ' + state.message));
    }
    if (state is Finished) {
      return Center(child: Text('Imported ${state.count} assets'));
    }
    return Container();
  }
}
