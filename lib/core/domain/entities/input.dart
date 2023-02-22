//
// Copyright (c) 2023 Nathan Fiedler
//
import 'package:equatable/equatable.dart';
import 'package:oxidized/oxidized.dart';

/// An `AssetInput` is ued to update an asset.
class AssetInput extends Equatable {
  // The list of tags associated with this asset.
  final List<String> tags;
  // A caption attributed to the asset.
  final Option<String> caption;
  // Location information for the asset.
  final Option<String> location;
  // The date/time that best represents the asset.
  final Option<DateTime> datetime;
  // The media type (nee MIME type) of the asset.
  final Option<String> mimetype;
  // The original filename of the asset when it was imported.
  final Option<String> filename;

  const AssetInput({
    required this.tags,
    required this.caption,
    required this.location,
    required this.datetime,
    required this.mimetype,
    required this.filename,
  });

  @override
  List<Object> get props => [filename];

  @override
  bool get stringify => true;
}

/// An `AssetInputId` is composed of an identifier and asset input.
class AssetInputId extends Equatable {
  final String id;
  final AssetInput input;

  const AssetInputId({required this.id, required this.input});

  @override
  List<Object> get props => [id];

  @override
  bool get stringify => true;
}
