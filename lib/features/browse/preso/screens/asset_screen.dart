//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:intl/intl.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/environment_config.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_bloc.dart';
import 'package:tanuki/container.dart';

class AssetScreen extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    final String assetId = ModalRoute.of(context).settings.arguments;
    return BlocProvider<AssetBloc>(
      create: (_) => getIt<AssetBloc>(),
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
                title: Text('Details for ${state.asset.filename}'),
                actions: [
                  FlatButton(
                    child: Icon(Icons.edit),
                    onPressed: () {
                      // replace the route for viewing the asset
                      Navigator.pushReplacementNamed(context, '/edit',
                          arguments: state.asset.id);
                    },
                  ),
                ],
              ),
              body: AssetPreview(asset: state.asset),
            );
          }
          return Center(child: CircularProgressIndicator());
        },
      ),
    );
  }
}

const thumbnail640 = '/api/thumbnail/640/640/';

class AssetPreview extends StatelessWidget {
  final Asset asset;

  AssetPreview({
    Key key,
    @required this.asset,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final uri = '${EnvironmentConfig.base_url}$thumbnail640${asset.id}';
    return SingleChildScrollView(
      child: Column(
        children: [
          Padding(
            padding: const EdgeInsets.all(8.0),
            child: Image.network(uri, fit: BoxFit.contain),
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
  final GlobalKey<FormBuilderState> _fbKey = GlobalKey<FormBuilderState>();

  @override
  Widget build(BuildContext context) {
    final datefmt = DateFormat.yMd().add_jm();
    //
    // Other asset properties not shown here:
    //
    // - asset.id
    // - asset.checksum
    // - asset.userdate
    //
    return FormBuilder(
      key: _fbKey,
      initialValue: {
        'datetime': datefmt.format(widget.asset.datetime.toLocal()),
        'filename': widget.asset.filename,
        'filesize': widget.asset.filesize.toString(),
        'mimetype': widget.asset.mimetype,
        'tags': widget.asset.tags.join(', '),
        'caption': widget.asset.caption.unwrapOr('(none)'),
        'location': widget.asset.location.unwrapOr('(none)'),
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
          FormBuilderTextField(
            attribute: 'caption',
            decoration: InputDecoration(
              icon: Icon(Icons.format_quote),
              labelText: 'Caption',
            ),
            readOnly: true,
          ),
          FormBuilderTextField(
            attribute: 'tags',
            decoration: InputDecoration(
              icon: Icon(Icons.label),
              labelText: 'Tags',
            ),
            readOnly: true,
          ),
          FormBuilderTextField(
            attribute: 'location',
            decoration: InputDecoration(
              icon: Icon(Icons.location_on),
              labelText: 'Location',
            ),
            readOnly: true,
          ),
          FormBuilderTextField(
            attribute: 'filename',
            decoration: InputDecoration(
              icon: Icon(Icons.folder_outlined),
              labelText: 'File name',
            ),
            readOnly: true,
          ),
          FormBuilderTextField(
            attribute: 'filesize',
            decoration: InputDecoration(
              icon: Icon(Icons.info_outline),
              labelText: 'File size',
            ),
            readOnly: true,
          ),
          FormBuilderTextField(
            attribute: 'mimetype',
            decoration: InputDecoration(
              icon: Icon(Icons.code),
              labelText: 'Media type',
            ),
            readOnly: true,
          ),
        ],
      ),
    );
  }
}
