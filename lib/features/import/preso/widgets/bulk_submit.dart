//
// Copyright (c) 2022 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tanuki/core/domain/entities/input.dart';
import 'package:tanuki/features/import/preso/bloc/bulk_update_bloc.dart';
import 'package:tanuki/features/import/preso/bloc/providers.dart';

typedef BulkCallback = List<AssetInputId> Function();

class BulkSubmit extends ConsumerWidget {
  final bool enabled;
  final BulkCallback onSubmit;
  final VoidCallback onComplete;

  const BulkSubmit({
    Key? key,
    required this.enabled,
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
          return ElevatedButton(
            onPressed: enabled
                ? () {
                    final inputs = onSubmit();
                    if (inputs.isNotEmpty) {
                      BlocProvider.of<BulkUpdateBloc>(context).add(
                        SubmitUpdates(inputs: inputs),
                      );
                    }
                  }
                : null,
            child: const Text('SAVE'),
          );
        },
      ),
    );
  }
}
