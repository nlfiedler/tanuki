//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:equatable/equatable.dart';
import 'package:oxidized/oxidized.dart';

/// The location for an asset.
class AssetLocation extends Equatable {
  // User-provided label for the location.
  final Option<String> label;
  // City associated with the asset.
  final Option<String> city;
  // State or province associated with the asset.
  final Option<String> region;

  const AssetLocation({
    required this.label,
    required this.city,
    required this.region,
  });

  factory AssetLocation.from(String? label) {
    return AssetLocation(
      label: Option.from(label),
      city: const None(),
      region: const None(),
    );
  }

  /// Return a user-visible description for this location.
  String description() {
    final hasLabel = label.isSome();
    final hasCity = city.isSome();
    final hasRegion = region.isSome();
    if (hasLabel && hasCity && hasRegion) {
      return "${label.unwrap()} - ${city.unwrap()}, ${region.unwrap()}";
    } else if (hasCity && hasRegion) {
      return "${city.unwrap()}, ${region.unwrap()}";
    } else if (hasLabel) {
      return label.unwrap();
    } else if (hasCity) {
      return city.unwrap();
    } else if (hasRegion) {
      return region.unwrap();
    }
    return "";
  }

  @override
  List<Object> get props => [label, city, region];

  @override
  bool get stringify => true;
}

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
  final Option<AssetLocation> location;

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
