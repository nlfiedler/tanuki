//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:io';

import 'package:file_chooser/file_chooser.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:tanuki/container.dart';
import 'package:tanuki/features/import/preso/bloc/upload_file_bloc.dart';

class UploadForm extends StatefulWidget {
  @override
  _UploadFormState createState() => _UploadFormState();
}

class _UploadFormState extends State<UploadForm> {
  List<String> _selectedFiles = [];

  void _pickFiles(BuildContext context) async {
    final FileChooserResult results = await showOpenPanel(
      allowsMultipleSelection: true,
    );
    if (!results.canceled) {
      setState(() {
        _selectedFiles.addAll(results.paths);
      });
    }
  }

  Widget _buildUploadStatus(BuildContext context, UploadFileState state) {
    if (state is Error) {
      return Text('Upload error: ' + state.message);
    }
    if (state is Uploading) {
      return Text('Uploading ${state.current}...');
    }
    if (_selectedFiles.isNotEmpty) {
      return Text('Use the Upload button to upload the files.');
    }
    if (state is Finished) {
      if (state.skipped.isNotEmpty) {
        return Column(
          children: [
            Text('The following files could not be copied:'),
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
      return Text('All done!');
    }
    return Text('Use the Choose Files button to get started.');
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
      final reader = File(uploading);
      final contents = await reader.readAsBytes();
      BlocProvider.of<UploadFileBloc>(context).add(
        UploadFile(
          filename: uploading,
          contents: contents,
        ),
      );
    } catch (e) {
      Scaffold.of(context).showSnackBar(
        SnackBar(
          content: ListTile(
            title: Text('Error reading file ${uploading}'),
            subtitle: Text(e.toString()),
          ),
        ),
      );
      BlocProvider.of<UploadFileBloc>(context).add(SkipCurrent());
    }
  }

  @override
  Widget build(BuildContext context) {
    return BlocProvider<UploadFileBloc>(
      create: (_) => getIt<UploadFileBloc>(),
      child: BlocConsumer<UploadFileBloc, UploadFileState>(
        listener: (context, state) async {
          if (state is Uploading) {
            await _uploadFile(context, state.current);
          }
        },
        builder: (context, state) {
          return Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisAlignment: MainAxisAlignment.spaceAround,
            children: <Widget>[
              Padding(
                padding: const EdgeInsets.fromLTRB(96.0, 48.0, 16.0, 16.0),
                child: RaisedButton(
                  onPressed: () => _pickFiles(context),
                  child: Text('Choose Files'),
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
                child: RaisedButton(
                  onPressed: _selectedFiles.isNotEmpty
                      ? () => _startUpload(context)
                      : null,
                  child: Text('Upload'),
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
