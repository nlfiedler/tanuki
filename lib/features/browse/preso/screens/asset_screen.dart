//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:intl/intl.dart';
import 'package:responsive_framework/responsive_framework.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_bloc.dart';
import 'package:tanuki/features/browse/preso/bloc/providers.dart';
import 'package:tanuki/features/browse/preso/widgets/asset_preview_visual.dart';

class AssetScreen extends ConsumerWidget {
  const AssetScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final assetId = ModalRoute.of(context)?.settings.arguments;
    if (assetId == null) {
      // hot restart seems to blow away the arguments, so show a button to
      // navigate back to the home screen
      return Center(
        child: ElevatedButton(
          child: const Text('GO BACK'),
          onPressed: () => Navigator.pop(context),
        ),
      );
    } else {
      return BlocProvider<AssetBloc>(
        create: (_) => ref.read(assetBlocProvider),
        child: BlocBuilder<AssetBloc, AssetState>(
          buildWhen: (previous, current) {
            return !(previous is Loaded && current is Loading);
          },
          builder: (context, state) {
            if (state is Empty) {
              // kick off the initial remote request
              BlocProvider.of<AssetBloc>(context)
                  .add(LoadAsset(id: assetId as String));
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
                    IconButton(
                      onPressed: () {
                        // replace the route for viewing the asset
                        Navigator.pushReplacementNamed(context, '/edit',
                            arguments: state.asset.id);
                      },
                      icon: const Icon(Icons.edit),
                      tooltip: 'Edit details',
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
}

class AssetPreview extends StatelessWidget {
  final Asset asset;

  const AssetPreview({super.key, required this.asset});

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      child: Column(
        children: [
          Padding(
            padding: const EdgeInsets.all(8.0),
            child: AssetPreviewVisual(asset: asset),
          ),
          Padding(
            padding: const EdgeInsets.fromLTRB(16.0, 8.0, 16.0, 32.0),
            child: AssetPreviewForm(asset: asset),
          ),
        ],
      ),
    );
  }
}

class AssetPreviewForm extends StatefulWidget {
  final Asset asset;

  const AssetPreviewForm({
    Key? key,
    required this.asset,
  }) : super(key: key);

  @override
  State<AssetPreviewForm> createState() => _AssetPreviewFormState();
}

class _AssetPreviewFormState extends State<AssetPreviewForm> {
  final GlobalKey<FormBuilderState> _fbKey = GlobalKey<FormBuilderState>();
  final datefmt = DateFormat.yMd().add_jm();
  final sizefmt = NumberFormat();

  @override
  Widget build(BuildContext context) {
    final location = widget.asset.location.mapOr((e) => e.description(), '');
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
        'filepath': widget.asset.filepath,
        'filesize': sizefmt.format(widget.asset.filesize),
        'mediaType': widget.asset.mediaType,
        'tags': widget.asset.tags.join(', '),
        'caption': widget.asset.caption.unwrapOr(''),
        'location': location,
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
              labelText: 'Filename',
            ),
            readOnly: true,
          ),
          FormBuilderTextField(
            name: 'filesize',
            decoration: const InputDecoration(
              icon: Icon(Icons.info_outline),
              labelText: 'File Size',
            ),
            readOnly: true,
          ),
          FormBuilderTextField(
            name: 'mediaType',
            decoration: const InputDecoration(
              icon: Icon(Icons.code),
              labelText: 'Media Type',
            ),
            readOnly: true,
          ),
          FormBuilderTextField(
            name: 'filepath',
            decoration: const InputDecoration(
              icon: Icon(Icons.photo_library),
              labelText: 'Asset Path',
            ),
            readOnly: true,
          ),
        ],
      ),
    );
  }
}
