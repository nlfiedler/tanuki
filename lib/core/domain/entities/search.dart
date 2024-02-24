//
// Copyright (c) 2024 Nathan Fiedler
//
import 'package:equatable/equatable.dart';
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/asset.dart';

enum SortField { date, identifier, filename, mediaType }

enum SortOrder { ascending, descending }

/// `SearchParams` are used to query assets in the database.
class SearchParams extends Equatable {
  /// Tags that an asset should have. All should match.
  final List<String> tags;

  /// Locations of an asset. At least one must match.
  final List<String> locations;

  /// Date for filtering asset results. Only those assets whose canonical date
  /// occurs on or after this date will be returned.
  final Option<DateTime> after;

  /// Date for filtering asset results. Only those assets whose canonical date
  /// occurs before this date will be returned.
  final Option<DateTime> before;

  /// Find assets whose filename (e.g. img_3011.jpg) matches the one given.
  final Option<String> filename;

  /// Find assets whose media type (e.g. image/jpeg) matches the one given.
  final Option<String> mediaType;

  /// Field by which to sort the results.
  final Option<SortField> sortField;

  /// Order by which to sort the results.
  final Option<SortOrder> sortOrder;

  const SearchParams({
    this.tags = const [],
    this.locations = const [],
    this.after = const None(),
    this.before = const None(),
    this.filename = const None(),
    this.mediaType = const None(),
    this.sortField = const None(),
    this.sortOrder = const None(),
  });

  @override
  List<Object> get props => [
        tags,
        locations,
        after,
        before,
        filename,
        mediaType,
      ];

  @override
  bool get stringify => true;
}

/// A `SearchResult` holds the results of an asset search.
class SearchResult extends Equatable {
  /// Unique idenitifer of the asset.
  final String id;

  /// Original filename for the asset.
  final String filename;

  /// Media type (formerly MIME type) of the asset.
  final String mediaType;

  /// Location for the asset, if available.
  final Option<AssetLocation> location;

  /// The date/time for the matching asset.
  final DateTime datetime;

  const SearchResult({
    required this.id,
    required this.filename,
    required this.mediaType,
    required this.location,
    required this.datetime,
  });

  @override
  List<Object> get props => [id];

  @override
  bool get stringify => true;
}

/// `QueryResults` holds a set of search results returned from a database query,
/// along with the count of the total number of matching entities.
class QueryResults extends Equatable {
  /// List of results from query, possibly paginated.
  final List<SearchResult> results;

  /// Number of results overall matching the query.
  final int count;

  const QueryResults({
    required this.results,
    required this.count,
  });

  @override
  List<Object> get props => [count, results.length];

  @override
  bool get stringify => true;
}
