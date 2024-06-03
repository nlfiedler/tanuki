//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/asset.dart';

class AssetLocationModel extends AssetLocation {
  const AssetLocationModel({
    required Option<String> label,
    required Option<String> city,
    required Option<String> region,
  }) : super(
          label: label,
          city: city,
          region: region,
        );

  factory AssetLocationModel.from(AssetLocation location) {
    return AssetLocationModel(
      label: location.label,
      city: location.city,
      region: location.region,
    );
  }

  factory AssetLocationModel.fromJson(Map<String, dynamic> json) {
    final Option<String> label = Option.from(json['label']);
    final Option<String> city = Option.from(json['city']);
    final Option<String> region = Option.from(json['region']);
    return AssetLocationModel(
      label: label,
      city: city,
      region: region,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'label': label.toNullable(),
      'city': city.toNullable(),
      'region': region.toNullable(),
    };
  }
}

class AssetModel extends Asset {
  const AssetModel({
    required String id,
    required String checksum,
    required String filename,
    required String filepath,
    required int filesize,
    required DateTime datetime,
    required String mediaType,
    required List<String> tags,
    required Option<DateTime> userdate,
    required Option<String> caption,
    required Option<AssetLocation> location,
  }) : super(
          id: id,
          checksum: checksum,
          filename: filename,
          filepath: filepath,
          filesize: filesize,
          datetime: datetime,
          mediaType: mediaType,
          tags: tags,
          userdate: userdate,
          caption: caption,
          location: location,
        );

  factory AssetModel.from(Asset asset) {
    return AssetModel(
      id: asset.id,
      checksum: asset.checksum,
      filename: asset.filename,
      filepath: asset.filepath,
      filesize: asset.filesize,
      datetime: asset.datetime,
      mediaType: asset.mediaType,
      tags: asset.tags,
      userdate: asset.userdate,
      caption: asset.caption,
      location: asset.location,
    );
  }

  factory AssetModel.fromJson(Map<String, dynamic> json) {
    final List<String> tags = List.from(json['tags'].map((t) => t.toString()));
    final Option<String> caption = Option.from(json['caption']);
    final Option<AssetLocation> location = Option.from(json['location'])
        .map((v) => AssetLocationModel.fromJson(v as Map<String, dynamic>));
    final datetime = DateTime.parse(json['datetime']);
    final userdate = Option.from(json['userdate']).map(
      (v) => DateTime.parse(v as String),
    );
    return AssetModel(
      id: json['id'],
      checksum: json['checksum'],
      filename: json['filename'],
      filepath: json['filepath'],
      // limiting file size to 2^53 (in JavaScript) is acceptable
      filesize: int.parse(json['filesize']),
      datetime: datetime,
      mediaType: json['mediaType'],
      tags: tags,
      userdate: userdate,
      caption: caption,
      location: location,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'checksum': checksum,
      'filename': filename,
      'filepath': filepath,
      'filesize': filesize.toString(),
      'datetime': datetime.toIso8601String(),
      'mediaType': mediaType,
      'tags': tags,
      'userdate': userdate.mapOr((v) => v.toIso8601String(), null),
      'caption': caption.toNullable(),
      'location': location.mapOr(
        (v) => AssetLocationModel.from(v).toJson(),
        null,
      ),
    };
  }
}
