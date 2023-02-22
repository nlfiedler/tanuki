//
// Copyright (c) 2023 Nathan Fiedler
//
import 'package:equatable/equatable.dart';
import 'package:oxidized/oxidized.dart';

/// An `Asset` holds information about a single asset.
class Asset extends Equatable {
  // The unique asset identifier.
  final String id;
  // Hash digest of the contents of the asset.
  final String checksum;
  // The original filename of the asset when it was imported.
  final String filename;
  // The size in bytes of the asset.
  final int filesize;
  // The date/time that best represents the asset.
  final DateTime datetime;
  // The media type (nee MIME type) of the asset.
  final String mimetype;
  // The list of tags associated with this asset.
  final List<String> tags;
  // The date provided by the user.
  final Option<DateTime> userdate;
  // A caption attributed to the asset.
  final Option<String> caption;
  // Location information for the asset.
  final Option<String> location;

  const Asset({
    required this.id,
    required this.checksum,
    required this.filename,
    required this.filesize,
    required this.datetime,
    required this.mimetype,
    required this.tags,
    required this.userdate,
    required this.caption,
    required this.location,
  });

  @override
  List<Object> get props => [id, checksum, filename];

  @override
  bool get stringify => true;
}
