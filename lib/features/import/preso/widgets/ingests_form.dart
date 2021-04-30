//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tanuki/features/import/preso/bloc/ingest_assets_bloc.dart';
import 'package:tanuki/features/import/preso/bloc/providers.dart';

class IngestsForm extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return BlocProvider<IngestAssetsBloc>(
      create: (_) => BuildContextX(context).read(ingestAssetsBlocProvider),
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
            children: <TextSpan>[
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
        SizedBox(width: 8.0),
        ElevatedButton(
          onPressed: () {
            BlocProvider.of<IngestAssetsBloc>(context).add(
              ProcessUploads(),
            );
          },
          child: Text('IMPORT'),
        ),
      ],
    );
  }

  Widget _buildStatusRow(BuildContext context, IngestAssetsState state) {
    if (state is Processing) {
      return Center(child: CircularProgressIndicator());
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
