//
// Copyright (c) 2020 Nathan Fiedler
//
// ignore: avoid_web_libraries_in_flutter
import 'dart:html';
import 'dart:math' as math;
import 'dart:typed_data';
import 'package:file_picker_web/file_picker_web.dart';
import 'package:flutter_dropzone/flutter_dropzone.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:tanuki/container.dart';
import 'package:tanuki/features/import/preso/bloc/upload_file_bloc.dart';

class UploadForm extends StatefulWidget {
  @override
  _UploadFormState createState() => _UploadFormState();
}

class _UploadFormState extends State<UploadForm> {
  List<File> _selectedFiles = [];
  bool highlightDropZone = false;

  void _pickFiles(BuildContext context) async {
    final files = await FilePicker.getMultiFile();
    setState(() {
      _selectedFiles.addAll(files);
    });
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

  void _uploadFile(BuildContext context, File uploading) {
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
    // size of the headline text in the current them.
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
                    _selectedFiles.add(ev as File);
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

Widget _buildFileList(List<File> files, UploadFileState state) {
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

class DottedBorder extends StatelessWidget {
  final Color color;
  final double strokeWidth;
  final double gap;
  final Widget child;
  final EdgeInsets padding;

  DottedBorder({
    this.color = Colors.black,
    this.strokeWidth = 1.0,
    this.gap = 5.0,
    this.padding = const EdgeInsets.all(8.0),
    @required this.child,
  }) {
    assert(child != null);
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      child: Padding(
        padding: EdgeInsets.all(strokeWidth / 2),
        child: Stack(
          children: <Widget>[
            Positioned.fill(
              child: CustomPaint(
                painter: _DashRectPainter(
                  color: color,
                  strokeWidth: strokeWidth,
                  gap: gap,
                ),
              ),
            ),
            Padding(
              padding: padding,
              child: child,
            ),
          ],
        ),
      ),
    );
  }
}

class _DashRectPainter extends CustomPainter {
  double strokeWidth;
  Color color;
  double gap;

  _DashRectPainter({
    this.strokeWidth = 5.0,
    this.color = Colors.black,
    this.gap = 5.0,
  });

  @override
  void paint(Canvas canvas, Size size) {
    Paint dashedPaint = Paint()
      ..color = color
      ..strokeWidth = strokeWidth
      ..style = PaintingStyle.stroke;

    double x = size.width;
    double y = size.height;

    Path _topPath = getDashedPath(
      a: math.Point(0, 0),
      b: math.Point(x, 0),
      gap: gap,
    );

    Path _rightPath = getDashedPath(
      a: math.Point(x, 0),
      b: math.Point(x, y),
      gap: gap,
    );

    Path _bottomPath = getDashedPath(
      a: math.Point(0, y),
      b: math.Point(x, y),
      gap: gap,
    );

    Path _leftPath = getDashedPath(
      a: math.Point(0, 0),
      b: math.Point(0.001, y),
      gap: gap,
    );

    canvas.drawPath(_topPath, dashedPaint);
    canvas.drawPath(_rightPath, dashedPaint);
    canvas.drawPath(_bottomPath, dashedPaint);
    canvas.drawPath(_leftPath, dashedPaint);
  }

  Path getDashedPath({
    @required math.Point<double> a,
    @required math.Point<double> b,
    @required gap,
  }) {
    Size size = Size(b.x - a.x, b.y - a.y);
    Path path = Path();
    path.moveTo(a.x, a.y);
    bool shouldDraw = true;
    math.Point currentPoint = math.Point(a.x, a.y);

    num radians = math.atan(size.height / size.width);

    num dx = math.cos(radians) * gap < 0
        ? math.cos(radians) * gap * -1
        : math.cos(radians) * gap;

    num dy = math.sin(radians) * gap < 0
        ? math.sin(radians) * gap * -1
        : math.sin(radians) * gap;

    while (currentPoint.x <= b.x && currentPoint.y <= b.y) {
      shouldDraw
          ? path.lineTo(currentPoint.x, currentPoint.y)
          : path.moveTo(currentPoint.x, currentPoint.y);
      shouldDraw = !shouldDraw;
      currentPoint = math.Point(
        currentPoint.x + dx,
        currentPoint.y + dy,
      );
    }
    return path;
  }

  @override
  bool shouldRepaint(CustomPainter oldDelegate) {
    return true;
  }
}
