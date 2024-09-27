//
// Copyright (c) 2024 Nathan Fiedler
//
import 'dart:io';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tanuki/features/import/preso/bloc/upload_file_bloc.dart';
import 'package:tanuki/features/import/preso/bloc/providers.dart';

// ignore: use_key_in_widget_constructors
class UploadForm extends ConsumerStatefulWidget {
  @override
  ConsumerState<UploadForm> createState() => _UploadFormState();
}

class _UploadFormState extends ConsumerState<UploadForm> {
  List<String> _selectedFiles = [];

  void _pickFiles(BuildContext context) async {
    final files = await openFiles();
    setState(() {
      for (var entry in files) {
        _selectedFiles.add(entry.path);
      }
    });
  }

  Widget _buildUploadStatus(BuildContext context, UploadFileState state) {
    if (state is Error) {
      return Text('Upload error: ${state.message}');
    }
    if (state is Uploading) {
      var value = state.uploaded / (state.active + state.uploaded);
      return Row(
        children: [
          CircularProgressIndicator(value: value),
          const SizedBox(width: 16.0),
          Expanded(
            child: Text('Uploading ${state.current}...'),
          ),
        ],
      );
    }
    if (_selectedFiles.isNotEmpty) {
      return const Text('Use the Upload button to upload the files.');
    }
    if (state is Finished) {
      if (state.skipped.isNotEmpty) {
        return Column(
          children: [
            const Text('The following files could not be copied:'),
            Expanded(
              child: ListView.builder(
                itemBuilder: (BuildContext context, int index) {
                  return Text(state.skipped[index]);
                },
                itemCount: state.skipped.length,
              ),
            ),
          ],
        );
      }
      return const Text('All done!');
    }
    return const Text('Use the Choose Files button to get started.');
  }

  void _startUpload(BuildContext context) {
    BlocProvider.of<UploadFileBloc>(context).add(
      StartUploading(files: _selectedFiles),
    );
    setState(() {
      _selectedFiles = [];
    });
  }

  void _uploadFile(BuildContext context, String uploading) async {
    try {
      final provider = BlocProvider.of<UploadFileBloc>(context);
      final reader = File(uploading);
      final contents = await reader.readAsBytes();
      provider.add(
        UploadFile(
          filename: uploading,
          contents: contents,
        ),
      );
    } catch (e) {
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Error reading file $uploading: $e')),
        );
        BlocProvider.of<UploadFileBloc>(context).add(SkipCurrent());
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return BlocProvider<UploadFileBloc>(
      create: (_) => ref.read(uploadFileBlocProvider),
      child: BlocConsumer<UploadFileBloc, UploadFileState>(
        listener: (context, state) async {
          if (state is Uploading) {
            _uploadFile(context, state.current as String);
          }
        },
        builder: (context, state) {
          return Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisAlignment: MainAxisAlignment.spaceAround,
            children: <Widget>[
              Padding(
                padding: const EdgeInsets.fromLTRB(96.0, 48.0, 16.0, 16.0),
                child: ElevatedButton(
                  onPressed: () => _pickFiles(context),
                  child: const Text('Choose Files'),
                ),
              ),
              Expanded(
                child: Column(
                  children: [
                    Center(
                      child: Padding(
                        padding: const EdgeInsets.fromLTRB(
                          16.0,
                          48.0,
                          16.0,
                          16.0,
                        ),
                        child: _buildUploadStatus(context, state),
                      ),
                    ),
                    //
                    // drag-and-drop widget will eventually go here, once
                    // flutter desktop has support for that
                    //
                    // c.f. https://github.com/flutter/flutter/issues/30719
                    //
                    _buildFileList(_selectedFiles, state),
                  ],
                ),
              ),
              Padding(
                padding: const EdgeInsets.fromLTRB(16.0, 48.0, 96.0, 16.0),
                child: ElevatedButton(
                  onPressed: _selectedFiles.isNotEmpty
                      ? () => _startUpload(context)
                      : null,
                  child: const Text('Upload'),
                ),
              )
            ],
          );
        },
      ),
    );
  }
}

Widget _buildFileList(List<String> files, UploadFileState state) {
  if (files.isNotEmpty) {
    return _buildListView(files);
  } else if (state is Uploading) {
    return _buildListView(state.pending);
  } else {
    return Container();
  }
}

Widget _buildListView(List<dynamic> files) {
  return Expanded(
    child: ListView.builder(
      itemBuilder: (BuildContext context, int index) {
        return ListTile(title: Text(files[index]));
      },
      itemCount: files.length,
    ),
  );
}
