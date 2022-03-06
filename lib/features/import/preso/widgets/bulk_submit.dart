//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tanuki/core/domain/entities/input.dart';
import 'package:tanuki/features/import/preso/bloc/bulk_update_bloc.dart';
import 'package:tanuki/features/import/preso/bloc/providers.dart';

typedef BulkCallback = List<AssetInputId> Function();

class BulkSubmit extends ConsumerWidget {
  final BulkCallback onSubmit;
  final VoidCallback onComplete;

  BulkSubmit({
    Key? key,
    required this.onSubmit,
    required this.onComplete,
  }) : super(key: key);

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return BlocProvider<BulkUpdateBloc>(
      create: (_) => ref.read(bulkUpdateBlocProvider),
      child: BlocConsumer<BulkUpdateBloc, BulkUpdateState>(
        listener: (context, state) {
          if (state is Finished) {
            onComplete();
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(content: Text('Updated ${state.count} assets')),
            );
          } else if (state is Error) {
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(content: Text('Error: ${state.message}')),
            );
          }
        },
        builder: (context, state) {
          return Padding(
            padding: const EdgeInsets.all(8.0),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Text('Fill in some or all of the fields and then'),
                Padding(
                  padding: const EdgeInsets.fromLTRB(8.0, 0, 8.0, 0),
                  child: ElevatedButton(
                    onPressed: () {
                      final inputs = onSubmit();
                      if (inputs.isNotEmpty) {
                        BlocProvider.of<BulkUpdateBloc>(context).add(
                          SubmitUpdates(inputs: inputs),
                        );
                      }
                    },
                    child: Text('SAVE'),
                  ),
                ),
              ],
            ),
          );
        },
      ),
    );
  }
}
