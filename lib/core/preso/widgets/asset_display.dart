//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:tanuki/environment_config.dart';
import 'package:video_player/video_player.dart';

class AssetDisplay extends StatelessWidget {
  final String assetId;
  final String mimetype;
  final int displayWidth;

  AssetDisplay({
    Key key,
    @required this.assetId,
    @required this.mimetype,
    @required this.displayWidth,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final baseUrl = EnvironmentConfig.base_url;
    if (mimetype.startsWith('video/')) {
      final uri = '$baseUrl/api/asset/$assetId';
      return _AssetVideo(uri: uri);
    } else {
      final tail = '$displayWidth/$displayWidth/$assetId';
      final uri = '$baseUrl/api/thumbnail/$tail';
      return Image.network(
        uri,
        fit: BoxFit.contain,
        errorBuilder: _imageErrorBuilder,
      );
    }
  }
}

Widget _imageErrorBuilder(
  BuildContext context,
  Object error,
  StackTrace stackTrace,
) {
  return Center(
    child: Card(
      child: ListTile(
        leading: Icon(Icons.error_outline),
        title: Text('Unable to display asset'),
        subtitle: Text(error.toString()),
      ),
    ),
  );
}

class _AssetVideo extends StatefulWidget {
  final String uri;

  _AssetVideo({
    Key key,
    @required this.uri,
  }) : super(key: key);

  @override
  _AssetVideoState createState() => _AssetVideoState();
}

class _AssetVideoState extends State<_AssetVideo> {
  VideoPlayerController _controller;

  @override
  void initState() {
    // known to work (web): .mov .mp4 .ogg .webm
    // does not work (web): .avi .wmv
    super.initState();
    _controller = VideoPlayerController.network(widget.uri);
    _controller.addListener(() {
      setState(() {});
    });
    _controller.initialize().then((_) => setState(() {}));
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      child: Container(
        padding: const EdgeInsets.all(20),
        child: AspectRatio(
          aspectRatio: _controller.value.aspectRatio,
          child: Stack(
            alignment: Alignment.bottomCenter,
            children: <Widget>[
              VideoPlayer(_controller),
              _ControlsOverlay(controller: _controller),
              VideoProgressIndicator(_controller, allowScrubbing: true),
            ],
          ),
        ),
      ),
    );
  }
}

class _ControlsOverlay extends StatelessWidget {
  final VideoPlayerController controller;

  const _ControlsOverlay({Key key, this.controller}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: <Widget>[
        AnimatedSwitcher(
          duration: Duration(milliseconds: 50),
          reverseDuration: Duration(milliseconds: 200),
          child: controller.value.isPlaying
              ? SizedBox.shrink()
              : Container(
                  color: Colors.black26,
                  child: Center(
                    child: Icon(
                      Icons.play_arrow,
                      color: Colors.white,
                      size: 100.0,
                    ),
                  ),
                ),
        ),
        GestureDetector(
          onTap: () {
            controller.value.isPlaying ? controller.pause() : controller.play();
          },
        ),
      ],
    );
  }
}
