//
// Copyright (c) 2020 Nathan Fiedler
//
// ignore: avoid_web_libraries_in_flutter
import 'dart:html';
import 'dart:typed_data';
import 'package:file_picker_web/file_picker_web.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:tanuki/container.dart';
import 'package:tanuki/features/upload/preso/bloc/upload_file_bloc.dart';

class UploadScreen extends StatefulWidget {
  @override
  _UploadScreenState createState() => _UploadScreenState();
}

class _UploadScreenState extends State<UploadScreen> {
  List<File> _selectedFiles = [];

  void _pickFiles(BuildContext context) async {
    final files = await FilePicker.getMultiFile();
    setState(() {
      _selectedFiles = files;
    });
  }

  Widget buildUploadStatus(BuildContext context, UploadFileState state) {
    if (state is Error) {
      return Text('Upload error: ' + state.message);
    }
    if (state is Uploading) {
      return Text('Uploading ${state.current.name}...');
    }
    if (_selectedFiles.isNotEmpty) {
      return Text('Use the Upload button to upload the files.');
    }
    if (state is Finished) {
      if (state.skipped.isNotEmpty) {
        return Column(
          children: [
            Text('The following files could not be copied:'),
            ListView.builder(
              shrinkWrap: true,
              itemBuilder: (BuildContext context, int index) {
                return Text(state.skipped[index].name);
              },
              itemCount: state.skipped.length,
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

  void _uploadPending(BuildContext context, File uploading) {
    // It is easier to manage the callbacks here in the widgets than for the
    // bloc to manage this in response to events coming from the widgets.
    FileReader reader = FileReader();
    reader.onLoadEnd.listen((_) {
      final Uint8List contents = reader.result;
      BlocProvider.of<UploadFileBloc>(context).add(
        UploadFile(
          filename: uploading.name,
          contents: contents,
        ),
      );
    });
    reader.onError.listen((_) {
      final String errorMsg = reader.error.message;
      Scaffold.of(context).showSnackBar(
        SnackBar(
          content: ListTile(
            title: Text('Error reading file ${uploading.name}'),
            subtitle: Text(errorMsg),
          ),
        ),
      );
      BlocProvider.of<UploadFileBloc>(context).add(SkipCurrent());
    });
    reader.readAsArrayBuffer(uploading);
  }

  @override
  Widget build(BuildContext context) {
    return BlocProvider<UploadFileBloc>(
      create: (_) => getIt<UploadFileBloc>(),
      child: BlocConsumer<UploadFileBloc, UploadFileState>(
        listener: (context, state) {
          if (state is Uploading) {
            _uploadPending(context, state.current);
          }
        },
        builder: (context, state) {
          return Scaffold(
            appBar: AppBar(
              title: Text('all your assets will belong to us'),
            ),
            body: Row(
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
                          child: buildUploadStatus(context, state),
                        ),
                      ),
                      buildFileList(_selectedFiles, state),
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
            ),
          );
        },
      ),
    );
  }
}

Widget buildFileList(List<File> files, UploadFileState state) {
  if (files.isNotEmpty) {
    return Column(
      children: [
        Text('Selected files...'),
        buildListView(files),
      ],
    );
  } else {
    if (state is Uploading) {
      return Column(
        children: [
          Text('Pending files...'),
          buildListView(state.pending),
        ],
      );
    } else {
      return Container();
    }
  }
}

Widget buildListView(List<dynamic> files) {
  return ListView.builder(
    shrinkWrap: true,
    itemBuilder: (BuildContext context, int index) {
      return ListTile(title: Text(files[index].name));
    },
    itemCount: files.length,
  );
}
