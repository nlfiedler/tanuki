//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:intl/intl.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/preso/widgets/asset_display.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_bloc.dart';
import 'package:tanuki/features/browse/preso/bloc/providers.dart';
import 'package:tanuki/features/modify/preso/validators/media_type.dart';
import 'package:tanuki/features/modify/preso/widgets/update_submit.dart';

final GlobalKey<FormBuilderState> _fbKey = GlobalKey<FormBuilderState>();

class EditAssetScreen extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    // fetch the asset again just in case of concurrent edits
    final String assetId = ModalRoute.of(context).settings.arguments;
    return BlocProvider<AssetBloc>(
      create: (_) => BuildContextX(context).read(assetBlocProvider),
      child: BlocBuilder<AssetBloc, AssetState>(
        buildWhen: (previous, current) {
          return !(previous is Loaded && current is Loading);
        },
        builder: (context, state) {
          if (state is Empty) {
            // kick off the initial remote request
            BlocProvider.of<AssetBloc>(context).add(LoadAsset(id: assetId));
          }
          if (state is Error) {
            return Text('Error: ' + state.message);
          }
          if (state is Loaded) {
            return Scaffold(
              appBar: AppBar(
                title: Text('Editing ${state.asset.filename}'),
                actions: [
                  UpdateSubmit(assetId: assetId, formKey: _fbKey),
                ],
              ),
              body: AssetEditor(asset: state.asset),
            );
          }
          return Center(child: CircularProgressIndicator());
        },
      ),
    );
  }
}

class AssetEditor extends StatelessWidget {
  final Asset asset;

  AssetEditor({
    Key key,
    @required this.asset,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      child: Column(
        children: [
          Padding(
            padding: const EdgeInsets.all(8.0),
            child: AssetDisplay(
              assetId: asset.id,
              mimetype: asset.mimetype,
              displayWidth: 640,
            ),
          ),
          Padding(
            padding: const EdgeInsets.fromLTRB(16.0, 8.0, 16.0, 32.0),
            child: AssetEditForm(asset: asset),
          ),
        ],
      ),
    );
  }
}

class AssetEditForm extends StatefulWidget {
  final Asset asset;

  AssetEditForm({
    Key key,
    @required this.asset,
  }) : super(key: key);

  @override
  _AssetEditFormState createState() => _AssetEditFormState();
}

class _AssetEditFormState extends State<AssetEditForm> {
  final DateFormat datefmt = DateFormat.yMd().add_jm();

  @override
  Widget build(BuildContext context) {
    return FormBuilder(
      key: _fbKey,
      initialValue: {
        'datetime': datefmt.format(widget.asset.datetime.toLocal()),
        'userdate': widget.asset.userdate.unwrapOr(null),
        'mimetype': widget.asset.mimetype,
        'tags': widget.asset.tags.join(', '),
        'caption': widget.asset.caption.unwrapOr(''),
        'location': widget.asset.location.unwrapOr(''),
      },
      child: Column(
        children: [
          FormBuilderTextField(
            attribute: 'datetime',
            decoration: InputDecoration(
              icon: Icon(Icons.calendar_today),
              labelText: 'Date',
            ),
            readOnly: true,
          ),
          FormBuilderDateTimePicker(
            attribute: 'userdate',
            decoration: InputDecoration(
              icon: Icon(Icons.calendar_today),
              labelText: 'Custom Date',
            ),
            inputType: InputType.both,
          ),
          FormBuilderTextField(
            attribute: 'caption',
            decoration: InputDecoration(
              icon: Icon(Icons.format_quote),
              labelText: 'Caption',
            ),
          ),
          FormBuilderTextField(
            attribute: 'tags',
            decoration: InputDecoration(
              icon: Icon(Icons.label),
              labelText: 'Tags',
            ),
            valueTransformer: (text) {
              final List<String> tags = text.split(',');
              return List.of(
                tags.map((e) => e.trim()).where((e) => e.isNotEmpty),
              );
            },
          ),
          FormBuilderTextField(
            attribute: 'location',
            decoration: InputDecoration(
              icon: Icon(Icons.location_on),
              labelText: 'Location',
            ),
          ),
          FormBuilderTextField(
            attribute: 'mimetype',
            decoration: InputDecoration(
              icon: Icon(Icons.code),
              labelText: 'Media type',
            ),
            autovalidate: true,
            validators: [
              (val) {
                return validateMediaType(val.toString());
              },
            ],
          ),
        ],
      ),
    );
  }
}
