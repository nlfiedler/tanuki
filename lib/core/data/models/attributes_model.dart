//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:meta/meta.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';

class LocationModel extends Location {
  LocationModel({
    @required String label,
    @required int count,
  }) : super(
          label: label,
          count: count,
        );

  factory LocationModel.fromStore(Location location) {
    return LocationModel(
      label: location.label,
      count: location.count,
    );
  }

  factory LocationModel.fromJson(Map<String, dynamic> json) {
    return LocationModel(
      label: json['label'],
      count: json['count'],
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'label': label,
      'count': count,
    };
  }
}

class TagModel extends Tag {
  TagModel({
    @required String label,
    @required int count,
  }) : super(
          label: label,
          count: count,
        );

  factory TagModel.fromStore(Tag tag) {
    return TagModel(
      label: tag.label,
      count: tag.count,
    );
  }

  factory TagModel.fromJson(Map<String, dynamic> json) {
    return TagModel(
      label: json['label'],
      count: json['count'],
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'label': label,
      'count': count,
    };
  }
}

class YearModel extends Year {
  YearModel({
    @required String label,
    @required int count,
  }) : super(
          label: label,
          count: count,
        );

  factory YearModel.fromStore(Year year) {
    return YearModel(
      label: year.label,
      count: year.count,
    );
  }

  factory YearModel.fromJson(Map<String, dynamic> json) {
    return YearModel(
      label: json['label'],
      count: json['count'],
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'label': label,
      'count': count,
    };
  }
}
