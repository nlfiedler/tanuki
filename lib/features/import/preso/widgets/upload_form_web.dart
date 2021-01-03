//
// Copyright (c) 2020 Nathan Fiedler
//
// ignore: avoid_web_libraries_in_flutter
import 'dart:html';
import 'dart:typed_data';

import 'package:file_picker/file_picker.dart';
import 'package:flutter_dropzone/flutter_dropzone.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tanuki/core/preso/widgets/dotted_border.dart';
import 'package:tanuki/features/import/preso/bloc/upload_file_bloc.dart';
import 'package:tanuki/features/import/preso/bloc/providers.dart';

class UploadForm extends StatefulWidget {
  @override
  _UploadFormState createState() => _UploadFormState();
}

class _UploadFormState extends State<UploadForm> {
  // Selected files come from either the file picker or the drop zone,
  // and they have different types (PlatformFile vs dart::html::File).
  List<dynamic> _selectedFiles = [];
  bool highlightDropZone = false;

  void _pickFiles(BuildContext context) async {
    final result = await FilePicker.platform.pickFiles(allowMultiple: true);
    if (result != null) {
      setState(() {
        _selectedFiles.addAll(result.files);
      });
    }
  }

  Widget _buildUploadStatus(BuildContext context, UploadFileState state) {
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
            Expanded(
              child: ListView.builder(
                itemBuilder: (BuildContext context, int index) {
                  return Text(state.skipped[index].name);
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

  void _uploadFile(BuildContext context, dynamic uploading) {
    if (uploading is PlatformFile) {
      // file_picker 2.0 provides the file data as a property
      BlocProvider.of<UploadFileBloc>(context).add(
        UploadFile(
          filename: uploading.name,
          contents: uploading.bytes,
        ),
      );
    } else {
      // With the html files, it is easier to manage the callbacks here in the
      // widgets than for the bloc to manage this in response to events coming
      // from the widgets.
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
        ScaffoldMessenger.of(context).showSnackBar(
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
  }

  @override
  Widget build(BuildContext context) {
    return BlocProvider<UploadFileBloc>(
      create: (_) => BuildContextX(context).read(uploadFileBlocProvider),
      child: BlocConsumer<UploadFileBloc, UploadFileState>(
        listener: (context, state) {
          if (state is Uploading) {
            _uploadFile(context, state.current);
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
                    _buildDropZone(context),
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

  Widget _buildDropZone(BuildContext context) {
    final theme = Theme.of(context);
    final borderColor = highlightDropZone
        ? theme.colorScheme.secondary
        : theme.colorScheme.primary;
    // Instead of a hard-coded size for the drop zone, make it a factor of the
    // size of the headline text in the current theme.
    final boxHeight = theme.textTheme.headline1.fontSize;
    return DottedBorder(
      color: borderColor,
      strokeWidth: 1.0,
      gap: 4.0,
      child: Container(
        height: boxHeight,
        padding: EdgeInsets.all(8.0),
        child: Stack(
          children: [
            Builder(
              builder: (context) => DropzoneView(
                operation: DragOperation.copy,
                cursor: CursorType.grab,
                onHover: () {
                  if (!highlightDropZone) {
                    setState(() => highlightDropZone = true);
                  }
                },
                onLeave: () {
                  setState(() => highlightDropZone = false);
                },
                onDrop: (ev) {
                  // Even when dropping multiple files, this gets called once
                  // for each file in the set, so must append to the list.
                  setState(() {
                    _selectedFiles.add(ev);
                    highlightDropZone = false;
                  });
                },
              ),
            ),
            Center(child: Text('You can drag and drop files here')),
          ],
        ),
      ),
    );
  }
}

Widget _buildFileList(List<dynamic> files, UploadFileState state) {
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
        return ListTile(title: Text(files[index].name));
      },
      itemCount: files.length,
    ),
  );
}
