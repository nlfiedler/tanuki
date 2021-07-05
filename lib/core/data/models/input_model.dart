//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/input.dart';

class AssetInputModel extends AssetInput {
  AssetInputModel({
    required List<String> tags,
    required Option<String> caption,
    required Option<String> location,
    required Option<DateTime> datetime,
    required Option<String> mimetype,
    required Option<String> filename,
  }) : super(
          tags: tags,
          caption: caption,
          location: location,
          datetime: datetime,
          mimetype: mimetype,
          filename: filename,
        );

  factory AssetInputModel.from(AssetInput asset) {
    return AssetInputModel(
      tags: asset.tags,
      caption: asset.caption,
      location: asset.location,
      datetime: asset.datetime,
      mimetype: asset.mimetype,
      filename: asset.filename,
    );
  }

  factory AssetInputModel.fromJson(Map<String, dynamic> json) {
    final List<String> tags = List.from(json['tags'].map((t) => t.toString()));
    final Option<String> caption = Option.from(json['caption']);
    final Option<String> location = Option.from(json['location']);
    final Option<String> filename = Option.from(json['filename']);
    final Option<String> mimetype = Option.from(json['mimetype']);
    final datetime = Option.from(json['datetime']).map(
      (v) => DateTime.parse(v as String),
    );
    return AssetInputModel(
      tags: tags,
      caption: caption,
      location: location,
      datetime: datetime,
      mimetype: mimetype,
      filename: filename,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'tags': tags,
      'caption': caption.toNullable(),
      'location': location.toNullable(),
      'datetime': datetime.mapOr((v) => v.toIso8601String(), null),
      'mimetype': mimetype.toNullable(),
      'filename': filename.toNullable(),
    };
  }
}

class AssetInputIdModel extends AssetInputId {
  AssetInputIdModel({
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
