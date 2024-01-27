//
// Copyright (c) 2022 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:intl/intl.dart';
import 'package:responsive_framework/responsive_framework.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/preso/widgets/asset_display.dart';
import 'package:tanuki/environment_config.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_bloc.dart';
import 'package:tanuki/features/browse/preso/bloc/providers.dart';
import 'package:url_launcher/url_launcher_string.dart' as launcher;

class AssetScreen extends ConsumerWidget {
  const AssetScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final String assetId = ModalRoute.of(context)?.settings.arguments as String;
    return BlocProvider<AssetBloc>(
      create: (_) => ref.read(assetBlocProvider),
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
            return Text('Error: ${state.message}');
          }
          if (state is Loaded) {
            return Scaffold(
              appBar: AppBar(
                title: ResponsiveValue(
                  context,
                  defaultValue: Text('Details for ${state.asset.filename}'),
                  conditionalValues: [
                    Condition.smallerThan(
                      name: TABLET,
                      value: Text(state.asset.filename),
                    )
                  ],
                ).value,
                actions: [
                  TextButton(
                    onPressed: () async {
                      await downloadAsset(context, state.asset);
                    },
                    child: const Icon(Icons.file_download),
                  ),
                  TextButton(
                    onPressed: () {
                      // replace the route for viewing the asset
                      Navigator.pushReplacementNamed(context, '/edit',
                          arguments: state.asset.id);
                    },
                    child: const Icon(Icons.edit),
                  ),
                ],
              ),
              body: AssetPreview(asset: state.asset),
            );
          }
          return const Center(child: CircularProgressIndicator());
        },
      ),
    );
  }
}

Future<void> downloadAsset(BuildContext context, Asset asset) async {
  const baseUrl = EnvironmentConfig.base_url;
  // Use url_launcher_string since it is difficult to create a Uri that refers
  // to the host of the current web page; some day need to fix this properly.
  final url = '$baseUrl/api/asset/${asset.id}';
  final messenger = ScaffoldMessenger.of(context);
  if (await launcher.canLaunchUrlString(url)) {
    await launcher.launchUrlString(url);
  } else {
    messenger.showSnackBar(
      const SnackBar(content: Text('Could not launch URL')),
    );
  }
}

class AssetPreview extends StatelessWidget {
  final Asset asset;

  const AssetPreview({
    Key? key,
    required this.asset,
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

  const AssetEditForm({
    Key? key,
    required this.asset,
  }) : super(key: key);

  @override
  // ignore: library_private_types_in_public_api
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
            name: 'datetime',
            decoration: const InputDecoration(
              icon: Icon(Icons.calendar_today),
              labelText: 'Date',
            ),
            readOnly: true,
          ),
          FormBuilderTextField(
            name: 'caption',
            decoration: const InputDecoration(
              icon: Icon(Icons.format_quote),
              labelText: 'Caption',
            ),
            readOnly: true,
          ),
          FormBuilderTextField(
            name: 'tags',
            decoration: const InputDecoration(
              icon: Icon(Icons.label),
              labelText: 'Tags',
            ),
            readOnly: true,
          ),
          FormBuilderTextField(
            name: 'location',
            decoration: const InputDecoration(
              icon: Icon(Icons.location_on),
              labelText: 'Location',
            ),
            readOnly: true,
          ),
          FormBuilderTextField(
            name: 'filename',
            decoration: const InputDecoration(
              icon: Icon(Icons.folder_outlined),
              labelText: 'File name',
            ),
            readOnly: true,
          ),
          FormBuilderTextField(
            name: 'filesize',
            decoration: const InputDecoration(
              icon: Icon(Icons.info_outline),
              labelText: 'File size',
            ),
            readOnly: true,
          ),
          FormBuilderTextField(
            name: 'mimetype',
            decoration: const InputDecoration(
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
