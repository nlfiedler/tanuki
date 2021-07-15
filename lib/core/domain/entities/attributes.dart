//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:equatable/equatable.dart';

/// A `Location` holds the label and count for a single location value.
class Location extends Equatable implements Comparable<Location> {
  // Label for the given attribute (e.g. "disney land").
  final String label;
  // Count of assets with this attribute.
  final int count;

  Location({
    required this.label,
    required this.count,
  });

  @override
  List<Object> get props => [label, count];

  @override
  bool get stringify => true;

  @override
  int compareTo(Location other) {
    return label.compareTo(other.label);
  }
}

/// A `Tag` holds the label and count for a single tag value.
class Tag extends Equatable implements Comparable<Tag> {
  // Label for the given attribute (e.g. "kittens").
  final String label;
  // Count of assets with this attribute.
  final int count;

  Tag({
    required this.label,
    required this.count,
  });

  @override
  List<Object> get props => [label, count];

  @override
  bool get stringify => true;

  @override
  int compareTo(Tag other) {
    return label.compareTo(other.label);
  }
}

/// A `Year` holds the label and count for a single year value.
class Year extends Equatable implements Comparable<Year> {
  // Label for the given attribute (e.g. "2019").
  final String label;
  // Numeric value of the year (e.g. 2019).
  final int value;
  // Count of assets with this attribute.
  final int count;

  Year({
    required this.label,
    required this.count,
  }) : value = int.parse(label);

  @override
  List<Object> get props => [value, count];

  @override
  bool get stringify => true;

  @override
  int compareTo(Year other) {
    return value.compareTo(other.value);
  }
}
