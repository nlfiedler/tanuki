//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:oxidized/oxidized.dart';
import 'package:tanuki/core/domain/entities/search.dart';

class SearchParamsModel extends SearchParams {
  SearchParamsModel({
    List<String> tags = const [],
    List<String> locations = const [],
    Option<DateTime> after = const None(),
    Option<DateTime> before = const None(),
    Option<String> filename = const None(),
    Option<String> mimetype = const None(),
    Option<SortField> sortField = const None(),
    Option<SortOrder> sortOrder = const None(),
  }) : super(
          tags: tags,
          locations: locations,
          after: after,
          before: before,
          filename: filename,
          mimetype: mimetype,
          sortField: sortField,
          sortOrder: sortOrder,
        );

  factory SearchParamsModel.from(SearchParams params) {
    return SearchParamsModel(
      tags: params.tags,
      locations: params.locations,
      after: params.after,
      before: params.before,
      filename: params.filename,
      mimetype: params.mimetype,
      sortField: params.sortField,
      sortOrder: params.sortOrder,
    );
  }

  factory SearchParamsModel.fromJson(Map<String, dynamic> json) {
    final List<String> tags = json['tags'] == null
        ? []
        : List.from(json['tags'].map((t) => t.toString()));
    final List<String> locations = json['locations'] == null
        ? []
        : List.from(json['locations'].map((t) => t.toString()));
    final after = Option.from(json['after']).map(
      (v) => DateTime.parse(v as String),
    );
    final before = Option.from(json['before']).map(
      (v) => DateTime.parse(v as String),
    );
    final Option<String> filename = Option.from(json['filename']);
    final Option<String> mimetype = Option.from(json['mimetype']);
    final sortField = decodeSortField(json['sortField']);
    final sortOrder = decodeSortOrder(json['sortOrder']);
    return SearchParamsModel(
      tags: tags,
      locations: locations,
      after: after,
      before: before,
      filename: filename,
      mimetype: mimetype,
      sortField: sortField,
      sortOrder: sortOrder,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'tags': tags.isEmpty ? null : tags,
      'locations': locations.isEmpty ? null : locations,
      'after': after.mapOr((v) => v.toIso8601String(), null),
      'before': before.mapOr((v) => v.toIso8601String(), null),
      'filename': filename.toNullable(),
      'mimetype': mimetype.toNullable(),
      'sortField': encodeSortField(sortField),
      'sortOrder': encodeSortOrder(sortOrder),
    };
  }
}

Option<SortField> decodeSortField(String? field) {
  switch (field) {
    case 'DATE':
      return Some(SortField.date);
    case 'IDENTIFIER':
      return Some(SortField.identifier);
    case 'FILENAME':
      return Some(SortField.filename);
    case 'MEDIA_TYPE':
      return Some(SortField.mediaType);
    case 'LOCATION':
      return Some(SortField.location);
  }
  return None();
}

String? encodeSortField(Option<SortField> field) {
  return field.mapOr((SortField sf) {
    switch (sf) {
      case SortField.date:
        return 'DATE';
      case SortField.identifier:
        return 'IDENTIFIER';
      case SortField.filename:
        return 'FILENAME';
      case SortField.mediaType:
        return 'MEDIA_TYPE';
      case SortField.location:
        return 'LOCATION';
    }
  }, null);
}

Option<SortOrder> decodeSortOrder(String? order) {
  switch (order) {
    case 'ASCENDING':
      return Some(SortOrder.ascending);
    case 'DESCENDING':
      return Some(SortOrder.descending);
  }
  return None();
}

String? encodeSortOrder(Option<SortOrder> order) {
  return order.mapOr((SortOrder so) {
    switch (so) {
      case SortOrder.ascending:
        return 'ASCENDING';
      case SortOrder.descending:
        return 'DESCENDING';
    }
  }, null);
}

class SearchResultModel extends SearchResult {
  SearchResultModel({
    required String id,
    required String filename,
    required String mimetype,
    required Option<String> location,
    required DateTime datetime,
  }) : super(
          id: id,
          filename: filename,
          mimetype: mimetype,
          location: location,
          datetime: datetime,
        );

  factory SearchResultModel.fromResult(SearchResult result) {
    return SearchResultModel(
      id: result.id,
      filename: result.filename,
      mimetype: result.mimetype,
      location: result.location,
      datetime: result.datetime,
    );
  }

  factory SearchResultModel.fromJson(Map<String, dynamic> json) {
    return SearchResultModel(
      id: json['id'],
      filename: json['filename'],
      mimetype: json['mimetype'],
      location: Option.from(json['location']),
      datetime: DateTime.parse(json['datetime']),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'filename': filename,
      'mimetype': mimetype,
      'location': location.unwrapOr(''),
      'datetime': datetime.toIso8601String(),
    };
  }
}

class QueryResultsModel extends QueryResults {
  QueryResultsModel({
    required List<SearchResult> results,
    required int count,
  }) : super(results: results, count: count);

  factory QueryResultsModel.fromJson(Map<String, dynamic> json) {
    final List<SearchResultModel> results = List.from(
      json['results'].map((v) => SearchResultModel.fromJson(v)),
    );
    final count = json['count'];
    return QueryResultsModel(results: results, count: count);
  }

  Map<String, dynamic> toJson() {
    final results = List.from(
      this.results.map((v) => SearchResultModel.fromResult(v).toJson()),
    );
    return {'results': results, 'count': count};
  }
}
