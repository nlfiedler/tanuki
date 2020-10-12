//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/container.dart';
import 'package:tanuki/core/domain/entities/input.dart';
import 'package:tanuki/features/modify/preso/bloc/update_asset_bloc.dart';

class UpdateSubmit extends StatelessWidget {
  final GlobalKey<FormBuilderState> formKey;
  final String assetId;

  UpdateSubmit({
    Key key,
    @required this.assetId,
    @required this.formKey,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BlocProvider<UpdateAssetBloc>(
      create: (_) => getIt<UpdateAssetBloc>(),
      child: BlocConsumer<UpdateAssetBloc, UpdateAssetState>(
        listener: (context, state) {
          if (state is Finished) {
            Scaffold.of(context).showSnackBar(
              SnackBar(
                content: ListTile(
                  title: Text('Updated asset'),
                ),
              ),
            );
          } else if (state is Error) {
            Scaffold.of(context).showSnackBar(
              SnackBar(
                content: ListTile(
                  title: Text('Error updating asset'),
                  subtitle: Text(state.message),
                ),
              ),
            );
          }
        },
        builder: (context, state) {
          return RaisedButton(
            child: Text('SAVE'),
            onPressed: () {
              if (formKey.currentState.saveAndValidate()) {
                final input = buildAssetInputId(
                  assetId,
                  formKey.currentState.value,
                );
                BlocProvider.of<UpdateAssetBloc>(context).add(
                  SubmitUpdate(input: input),
                );
              }
            },
          );
        },
      ),
    );
  }
}

AssetInputId buildAssetInputId(String assetId, Map<String, dynamic> inputs) {
  // Undefined values are treated as empty string, so we can treat empty
  // and undefined in the same manner down below with Option.some().
  final String caption = inputs['caption'] ?? '';
  final String location = inputs['location'] ?? '';
  final DateTime datetime = inputs['userdate'] as DateTime;
  return AssetInputId(
    id: assetId,
    input: AssetInput(
      tags: inputs['tags'] as List<String>,
      caption: Option.some(caption.isEmpty ? null : caption),
      location: Option.some(location.isEmpty ? null : location),
      datetime: Option.some(datetime).map((v) => v.toUtc()),
      mimetype: Some(inputs['mimetype']),
      filename: None(),
    ),
  );
}
