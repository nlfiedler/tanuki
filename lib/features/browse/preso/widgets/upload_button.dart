//
// Copyright (c) 2024 Nathan Fiedler
//
// ignore: avoid_web_libraries_in_flutter
import 'dart:html';
import 'dart:typed_data';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tanuki/features/browse/preso/bloc/asset_bloc.dart' as ab;
import 'package:tanuki/features/browse/preso/bloc/asset_browser_bloc.dart'
    as abb;
import 'package:tanuki/features/browse/preso/bloc/providers.dart';
import 'package:tanuki/features/browse/preso/bloc/replace_file_bloc.dart';

class UploadButton extends ConsumerWidget {
  final String assetId;

  const UploadButton({super.key, required this.assetId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return BlocProvider<ReplaceFileBloc>(
      create: (_) => ref.read(replaceFileBlocProvider),
      child: BlocConsumer<ReplaceFileBloc, ReplaceFileState>(
        listener: (context, state) {
          if (state is Finished) {
            if (state.assetId == assetId) {
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(
                  content: Text('Identical asset already exists!'),
                ),
              );
            } else {
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(
                  content: Text('Replacement asset uploaded'),
                ),
              );
              // force the asset screen and preview to be refreshed, as well as
              // the browser which needs to get the new asset identifier
              BlocProvider.of<ab.AssetBloc>(context)
                  .add(ab.LoadAsset(id: state.assetId));
              BlocProvider.of<abb.AssetBrowserBloc>(context)
                  .add(abb.LoadInitialAssets());
            }
          } else if (state is Uploading) {
            _uploadFile(context, assetId, state.current);
          } else if (state is Error) {
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(content: Text('Error: ${state.message}')),
            );
          }
        },
        builder: (context, state) {
          return IconButton(
            onPressed: state is Uploading
                ? null
                : () async {
                    final file = await openFile();
                    if (file != null && context.mounted) {
                      BlocProvider.of<ReplaceFileBloc>(context).add(
                        StartUploading(file: file),
                      );
                    }
                  },
            icon: const Icon(Icons.file_upload),
            tooltip: 'Upload replacement asset',
          );
        },
      ),
    );
  }
}

void _uploadFile(
  BuildContext context,
  String assetId,
  dynamic uploading,
) async {
  // could be cross_file::XFile or dart::html::File
  if (uploading is XFile) {
    final provider = BlocProvider.of<ReplaceFileBloc>(context);
    final contents = await uploading.readAsBytes();
    provider.add(
      ReplaceFile(
        assetId: assetId,
        filename: uploading.name,
        contents: contents,
      ),
    );
  } else {
    // With the html files, it is easier to manage the callbacks here in the
    // widgets than for the bloc to manage this in response to events coming
    // from the widgets.
    FileReader reader = FileReader();
    reader.onLoadEnd.listen((_) {
      if (context.mounted) {
        final Uint8List contents = reader.result as Uint8List;
        BlocProvider.of<ReplaceFileBloc>(context).add(
          ReplaceFile(
            assetId: assetId,
            filename: uploading.name,
            contents: contents,
          ),
        );
      }
    });
    reader.onError.listen((_) {
      if (context.mounted) {
        final String errorMsg = reader.error?.message ?? '(none)';
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Error reading file ${uploading.name}: $errorMsg'),
          ),
        );
        BlocProvider.of<ReplaceFileBloc>(context).add(SkipCurrent());
      }
    });
    reader.readAsArrayBuffer(uploading);
  }
}
