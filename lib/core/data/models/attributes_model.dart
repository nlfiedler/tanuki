//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:meta/meta.dart';
import 'package:tanuki/core/domain/entities/attributes.dart';

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
