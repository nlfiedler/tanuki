//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/asset.dart';

class AssetModel extends Asset {
  AssetModel({
    required String id,
    required String checksum,
    required String filename,
    required int filesize,
    required DateTime datetime,
    required String mimetype,
    required List<String> tags,
    required Option<DateTime> userdate,
    required Option<String> caption,
    required Option<String> location,
  }) : super(
          id: id,
          checksum: checksum,
          filename: filename,
          filesize: filesize,
          datetime: datetime,
          mimetype: mimetype,
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
      filesize: asset.filesize,
      datetime: asset.datetime,
      mimetype: asset.mimetype,
      tags: asset.tags,
      userdate: asset.userdate,
      caption: asset.caption,
      location: asset.location,
    );
  }

  factory AssetModel.fromJson(Map<String, dynamic> json) {
    final List<String> tags = List.from(json['tags'].map((t) => t.toString()));
    final Option<String> caption = Option.from(json['caption']);
    final Option<String> location = Option.from(json['location']);
    final datetime = DateTime.parse(json['datetime']);
    final userdate = Option.from(json['userdate']).map(
      (v) => DateTime.parse(v as String),
    );
    return AssetModel(
      id: json['id'],
      checksum: json['checksum'],
      filename: json['filename'],
      // limiting file size to 2^53 (in JavaScript) is acceptable
      filesize: int.parse(json['filesize']),
      datetime: datetime,
      mimetype: json['mimetype'],
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
      'filesize': filesize.toString(),
      'datetime': datetime.toIso8601String(),
      'mimetype': mimetype,
      'tags': tags,
      'userdate': userdate.mapOr((v) => v.toIso8601String(), null),
      'caption': caption.toNullable(),
      'location': location.toNullable(),
    };
  }
}
