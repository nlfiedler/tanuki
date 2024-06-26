//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:oxidized/oxidized.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/domain/entities/input.dart';
import 'package:tanuki/features/modify/preso/bloc/update_asset_bloc.dart';
import 'package:tanuki/features/modify/preso/bloc/providers.dart';

class UpdateSubmit extends ConsumerWidget {
  final GlobalKey<FormBuilderState> formKey;
  final String assetId;

  const UpdateSubmit({
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
              const SnackBar(content: Text('Updated asset')),
            );
            Navigator.pushReplacementNamed(context, '/asset',
                arguments: state.asset.id);
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
            child: const Text('SAVE'),
          );
        },
      ),
    );
  }
}

AssetInputId buildAssetInputId(String assetId, Map<String, dynamic> inputs) {
  final Option<String> caption = Option.from(inputs['caption']);
  // empty fields will be sent to the backend as empty strings which should
  // instruct the backend to clear that field rather than ignoring it
  final Option<String> label = Option.from(inputs['location']);
  final Option<String> city = Option.from(inputs['city']);
  final Option<String> region = Option.from(inputs['region']);
  final Option<AssetLocation> location =
      label.isNone() && city.isNone() && region.isNone()
          ? const None()
          : Some(AssetLocation(label: label, city: city, region: region));
  final Option<DateTime> datetime = Option.from(inputs['userdate']?.toUtc());
  return AssetInputId(
    id: assetId,
    input: AssetInput(
      tags: inputs['tags'] as List<String>,
      caption: caption,
      location: location,
      datetime: datetime,
      mediaType: Some(inputs['mediaType']),
      filename: const None(),
    ),
  );
}
