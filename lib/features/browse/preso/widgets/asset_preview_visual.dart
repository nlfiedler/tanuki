//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:flutter/material.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/preso/widgets/asset_display.dart';
import 'package:tanuki/environment_config.dart';
import 'package:tanuki/features/browse/preso/widgets/upload_button.dart';
import 'package:url_launcher/url_launcher_string.dart' as launcher;

class AssetPreviewVisual extends StatelessWidget {
  final Asset asset;

  const AssetPreviewVisual({super.key, required this.asset});

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        Padding(
          padding: const EdgeInsets.all(8.0),
          child: IconButton(
            onPressed: () async {
              await downloadAsset(context, asset);
            },
            icon: const Icon(Icons.file_download),
            tooltip: 'Download asset',
          ),
        ),
        AssetDisplay(
          assetId: asset.id,
          mediaType: asset.mediaType,
          displayWidth: 640,
        ),
        Padding(
          padding: const EdgeInsets.all(8.0),
          child: UploadButton(assetId: asset.id),
        ),
      ],
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
