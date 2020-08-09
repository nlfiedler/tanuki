//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:equatable/equatable.dart';
import 'package:meta/meta.dart';

/// A `Year` holds the label and count for a single year value.
class Year extends Equatable {
  // Label for the given attribute (e..g "2019").
  final String label;
  // Count of assets with this attribute.
  final int count;

  Year({
    @required this.label,
    @required this.count,
  });

  @override
  List<Object> get props => [label, count];

  @override
  bool get stringify => true;
}

/// A `Location` holds the label and count for a single location value.
class Location extends Equatable {
  // Label for the given attribute (e.g. "disney land").
  final String label;
  // Count of assets with this attribute.
  final int count;

  Location({
    @required this.label,
    @required this.count,
  });

  @override
  List<Object> get props => [label, count];

  @override
  bool get stringify => true;
}
