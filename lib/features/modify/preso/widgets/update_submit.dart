//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:oxidized/oxidized.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tanuki/core/domain/entities/input.dart';
import 'package:tanuki/features/modify/preso/bloc/update_asset_bloc.dart';
import 'package:tanuki/features/modify/preso/bloc/providers.dart';

class UpdateSubmit extends ConsumerWidget {
  final GlobalKey<FormBuilderState> formKey;
  final String assetId;

  UpdateSubmit({
    Key? key,
    required this.assetId,
    required this.formKey,
  }) : super(key: key);

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return BlocProvider<UpdateAssetBloc>(
      create: (_) => ref.read(updateAssetBlocProvider),
      child: BlocConsumer<UpdateAssetBloc, UpdateAssetState>(
        listener: (context, state) {
          if (state is Finished) {
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(content: Text('Updated asset')),
            );
          } else if (state is Error) {
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(content: Text('Error: ${state.message}')),
            );
          }
        },
        builder: (context, state) {
          return ElevatedButton(
            onPressed: () {
              if (formKey.currentState!.saveAndValidate()) {
                final input = buildAssetInputId(
                  assetId,
                  formKey.currentState!.value,
                );
                BlocProvider.of<UpdateAssetBloc>(context).add(
                  SubmitUpdate(input: input),
                );
              }
            },
            child: Text('SAVE'),
          );
        },
      ),
    );
  }
}

AssetInputId buildAssetInputId(String assetId, Map<String, dynamic> inputs) {
  // Undefined values are treated as empty string, so we can treat empty
  // and undefined in the same manner down below with Option.from().
  final String caption = inputs['caption'] ?? '';
  final String location = inputs['location'] ?? '';
  final DateTime? datetime = inputs['userdate'] as DateTime;
  return AssetInputId(
    id: assetId,
    input: AssetInput(
      tags: inputs['tags'] as List<String>,
      caption: Option.from(caption.isEmpty ? null : caption),
      location: Option.from(location.isEmpty ? null : location),
      datetime: Option.from(datetime).map((v) => v.toUtc()),
      mimetype: Some(inputs['mimetype']),
      filename: None(),
    ),
  );
}
