//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/data/models/asset_model.dart';
import 'package:tanuki/core/domain/entities/asset.dart';
import 'package:tanuki/core/domain/entities/input.dart';

class AssetInputModel extends AssetInput {
  const AssetInputModel({
    required List<String> tags,
    required Option<String> caption,
    required Option<AssetLocation> location,
    required Option<DateTime> datetime,
    required Option<String> mediaType,
    required Option<String> filename,
  }) : super(
          tags: tags,
          caption: caption,
          location: location,
          datetime: datetime,
          mediaType: mediaType,
          filename: filename,
        );

  factory AssetInputModel.from(AssetInput asset) {
    return AssetInputModel(
      tags: asset.tags,
      caption: asset.caption,
      location: asset.location,
      datetime: asset.datetime,
      mediaType: asset.mediaType,
      filename: asset.filename,
    );
  }

  factory AssetInputModel.fromJson(Map<String, dynamic> json) {
    final List<String> tags = List.from(json['tags'].map((t) => t.toString()));
    final Option<String> caption = Option.from(json['caption']);
    final Option<AssetLocation> location = Option.from(json['location'])
        .map((v) => AssetLocationModel.fromJson(v as Map<String, dynamic>));
    final Option<String> filename = Option.from(json['filename']);
    final Option<String> mediaType = Option.from(json['mediaType']);
    final datetime = Option.from(json['datetime']).map(
      (v) => DateTime.parse(v as String),
    );
    return AssetInputModel(
      tags: tags,
      caption: caption,
      location: location,
      datetime: datetime,
      mediaType: mediaType,
      filename: filename,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'tags': tags,
      'caption': caption.toNullable(),
      'location': location.mapOr(
        (v) => AssetLocationModel.from(v).toJson(),
        null,
      ),
      'datetime': datetime.mapOr((v) => v.toIso8601String(), null),
      'mediaType': mediaType.toNullable(),
      'filename': filename.toNullable(),
    };
  }
}

class AssetInputIdModel extends AssetInputId {
  const AssetInputIdModel({
    required String id,
    required AssetInput input,
  }) : super(id: id, input: input);

  factory AssetInputIdModel.from(AssetInputId inputId) {
    return AssetInputIdModel(id: inputId.id, input: inputId.input);
  }

  factory AssetInputIdModel.fromJson(Map<String, dynamic> json) {
    final String id = json['id'];
    final AssetInput input = AssetInputModel.fromJson(json['input']);
    return AssetInputIdModel(id: id, input: input);
  }

  Map<String, dynamic> toJson() {
    final AssetInputModel innput = AssetInputModel.from(input);
    return {'id': id, 'input': innput.toJson()};
  }
}
