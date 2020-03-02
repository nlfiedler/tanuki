//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:angel_graphql/angel_graphql.dart';
import 'package:graphql_schema/graphql_schema.dart';
import 'package:graphql_server/graphql_server.dart';

final GraphQLScalarType<int, String> bigIntType = GraphQLBigIntType();

class GraphQLBigIntType extends GraphQLScalarType<int, String> {
  @override
  String get name => 'BigInt';

  @override
  String get description => 'A 64-bit signed integer value.';

  @override
  String serialize(int value) => value.toString();

  @override
  int deserialize(String serialized) => int.parse(serialized);

  @override
  ValidationResult<String> validate(String key, input) {
    if (input != null && input is! String) {
      return _Vr<String>(false,
          errors: ['$key must represent a signed 64-bit integer.']);
    } else if (input == null) {
      return _Vr<String>(true, value: input);
    }

    try {
      int.parse(input);
      return _Vr<String>(true, value: input);
    } on FormatException {
      return _Vr<String>(false,
          errors: ['$key must represent a signed 64-bit integer.']);
    }
  }

  @override
  GraphQLType<int, String> coerceToInputObject() => this;
}

// Temporary work-around for private constructors on ValidationResult.
class _Vr<T> implements ValidationResult<T> {
  final bool successful;
  final List<String> errors;
  final T value;

  _Vr(this.successful, {this.errors, this.value});
}

final GraphQLObjectType assetType = objectType(
  'Asset',
  description: 'Defines a single entity in the storage system.',
  fields: [
    field(
      'id',
      graphQLId.nonNullable(),
      resolve: (context, args) {
        return 'foobar';
      },
      description: 'The unique asset identifier.',
    ),
    field(
      'filename',
      graphQLString.nonNullable(),
      resolve: (context, args) {
        return 'filename';
      },
      description: 'The original filename of the asset when it was imported.',
    ),
    field(
      'filepath',
      graphQLString.nonNullable(),
      resolve: (context, args) {
        return 'file/path.ext';
      },
      description: 'Path of the asset in the storage directory structure.',
    ),
    field(
      'filesize',
      bigIntType.nonNullable(),
      // graphQLInt.nonNullable(),
      resolve: (context, args) {
        return 123456;
      },
      description: 'The size in bytes of the asset.',
    ),
    field(
      'datetime',
      graphQLDate.nonNullable(),
      resolve: (context, args) {
        return DateTime.now();
      },
      description: 'The date and time that best represents the asset.',
    ),
    field(
      'mimetype',
      graphQLString.nonNullable(),
      resolve: (context, args) {
        return 'application/octet-stream';
      },
      description: 'The media type of the asset, such as "image/jpeg".',
    ),
    field(
      'tags',
      listOf(graphQLString.nonNullable()).nonNullable(),
      resolve: (context, args) {
        return ['cat', 'dog', 'bird'];
      },
      description: 'The list of tags associated with this asset.',
    ),
    field(
      'userdate',
      graphQLDate,
      resolve: (context, args) {
        return null;
      },
      description: 'The date provided by the user.',
    ),
    field(
      'caption',
      graphQLString,
      resolve: (context, args) {
        return null;
      },
      description: 'A caption attributed to the asset.',
    ),
    field(
      'duration',
      graphQLFloat,
      resolve: (context, args) {
        return null;
      },
      description: 'For video assets, the duration in seconds.',
    ),
    field(
      'location',
      graphQLString,
      resolve: (context, args) {
        return null;
      },
      description: 'Location information for the asset.',
    ),
    field(
      'previewUrl',
      graphQLString.nonNullable(),
      resolve: (context, args) {
        return 'http://example.com/preview/asset.jpg';
      },
      description: 'URL for a smaller rendering of the asset image.',
    ),
    field(
      'assetUrl',
      graphQLString.nonNullable(),
      resolve: (context, args) {
        return 'http://example.com/full/asset.jpg';
      },
      description: 'URL of the full sized asset.',
    ),
  ],
);

final GraphQLObjectType tagCountType = objectType(
  'TagCount',
  description: 'Indicates the number of assets with a given tag.',
  fields: [
    field(
      'value',
      graphQLString.nonNullable(),
      resolve: (context, args) {
        return 'foobar';
      },
      description: 'The unique name of the tag.',
    ),
    field(
      'count',
      graphQLInt.nonNullable(),
      resolve: (context, args) {
        return 12;
      },
      description: 'The count of assets with this tag.',
    ),
  ],
);

final GraphQLObjectType yearCountType = objectType(
  'YearCount',
  description: 'Indicates the number of assets with a given year.',
  fields: [
    field(
      'value',
      graphQLInt.nonNullable(),
      resolve: (context, args) {
        return 2018;
      },
      description: 'The unique value for the year.',
    ),
    field(
      'count',
      graphQLInt.nonNullable(),
      resolve: (context, args) {
        return 12;
      },
      description: 'The count of assets with this year.',
    ),
  ],
);

final GraphQLObjectType locationCountType = objectType(
  'LocationCount',
  description: 'Indicates the number of assets with a given location.',
  fields: [
    field(
      'value',
      graphQLString.nonNullable(),
      resolve: (context, args) {
        return 'hawaii';
      },
      description: 'The unique value for the location.',
    ),
    field(
      'count',
      graphQLInt.nonNullable(),
      resolve: (context, args) {
        return 12;
      },
      description: 'The count of assets with this location.',
    ),
  ],
);

final GraphQLObjectType searchResultType = objectType(
  'SearchResult',
  description: 'Represents a single result from a query.',
  fields: [
    field(
      'id',
      graphQLId.nonNullable(),
      resolve: (context, args) {
        return 'foobar';
      },
      description: 'The identifier of the matching asset.',
    ),
    field(
      'datetime',
      graphQLDate.nonNullable(),
      resolve: (context, args) {
        return DateTime.now();
      },
      description: 'The date/time for the matching asset.',
    ),
    field(
      'filename',
      graphQLString.nonNullable(),
      resolve: (context, args) {
        return 'example.jpg';
      },
      description: 'The filename for the matching asset.',
    ),
    field(
      'location',
      graphQLString,
      resolve: (context, args) {
        return null;
      },
      description: 'The location for the matching asset, if available.',
    ),
    field(
      'thumbWidth',
      graphQLInt.nonNullable(),
      resolve: (context, args) {
        return 0;
      },
      description:
          'Pixel width of "wide" thumbnail (-1 if not an image, 0 if no thumbnail).',
    ),
    field(
      'thumbHeight',
      graphQLInt.nonNullable(),
      resolve: (context, args) {
        return 0;
      },
      description:
          'Pixel height of "wide" thumbnail (-1 if not an image, 0 if no thumbnail).',
    ),
    field(
      'thumbnailUrl',
      graphQLString.nonNullable(),
      resolve: (context, args) {
        return 'http://example.com';
      },
      description: 'Relative URL of the thumbnail fitting a 240x240 pixel box.',
    ),
    field(
      'widethumbUrl',
      graphQLString.nonNullable(),
      resolve: (context, args) {
        return 'http://example.com';
      },
      description:
          'Relative URL of the thumbnail fitting a 240xN pixel box (maintains aspect ratio).',
    ),
  ],
);

final GraphQLObjectType searchMetaType = objectType(
  'SearchMeta',
  description: 'Result from the search query.',
  fields: [
    field(
      'results',
      listOf(searchResultType.nonNullable()),
      resolve: (context, args) {
        return null;
      },
      description: 'The list of results retrieved via the query.',
    ),
    field(
      'count',
      graphQLInt.nonNullable(),
      resolve: (context, args) {
        return 0;
      },
      description: 'The total number of matching assets in the system.',
    ),
  ],
);

final GraphQLInputObjectType searchParamsType = inputObjectType(
  'SearchParams',
  description: 'Parameters by which to search for assets.',
  inputFields: [
    inputField(
      'tags',
      listOf(graphQLString),
      description: 'Tags of an asset. All must match.',
    ),
    inputField(
      'locations',
      listOf(graphQLString),
      description: 'Locations of an asset. At least one must match.',
    ),
    inputField(
      'after',
      graphQLDate,
      description: 'Select assets whose date/time is after this value.',
    ),
    inputField(
      'before',
      graphQLDate,
      description: 'Select assets whose date/time is before this value.',
    ),
    inputField(
      'filename',
      graphQLString,
      description: 'Select assets whose filename matches this value.',
    ),
    inputField(
      'mimetype',
      graphQLString,
      description: 'Select assets whose media type matches this value.',
    ),
  ],
);

final GraphQLObjectType queryType = objectType(
  'Query',
  description: 'Fields for querying the backend.',
  fields: [
    field(
      'asset',
      assetType,
      inputs: [
        GraphQLFieldInput('id', graphQLId.nonNullable()),
      ],
      resolve: (context, args) {
        return {
          'id': 'sha256-cafebabe',
          'filename': 'example.jpg',
          'filepath': 'somewhere/safe/here',
          'filesize': '1048576', // for BigInt, this must be a string
          'datetime': DateTime.now().toString(),
          'mimetype': 'image/jpeg',
          'tags': ['sam', 'larry', 'susan'],
          'userdate': null,
          'caption': 'having fun in the snow',
          'duration': null,
          'location': 'outside',
          'previewUrl': 'http://www.google.com',
          'assetUrl': 'http://flutter.dev'
        };
      },
      description: 'Retrieve an asset by its unique identifier.',
    ),
    field(
      'count',
      graphQLInt.nonNullable(),
      resolve: (context, args) {
        return 1234;
      },
      description: 'Return the total number of assets in the system.',
    ),
    field(
      'locations',
      listOf(locationCountType.nonNullable()).nonNullable(),
      resolve: (context, args) {
        return [
          {'value': 'hawaii', 'count': 12}
        ];
      },
      description:
          'Retrieve the list of locations and their associated asset count.',
    ),
    field(
      'lookup',
      assetType,
      inputs: [
        GraphQLFieldInput(
          'checksum',
          graphQLString.nonNullable(),
          description:
              'Hash digest with algorith prefix, such as "sha1-cafebabe".',
        ),
      ],
      resolve: (context, args) {
        return null;
      },
      description: 'Look up an asset by the hash digest of its content.',
    ),
    field(
      'search',
      searchMetaType.nonNullable(),
      inputs: [
        GraphQLFieldInput(
          'params',
          searchParamsType.nonNullable(),
        ),
        GraphQLFieldInput(
          'count',
          graphQLInt,
          defaultValue: 10,
        ),
        GraphQLFieldInput(
          'offset',
          graphQLInt,
          defaultValue: 0,
        ),
      ],
      resolve: (context, args) {
        return null;
      },
      description: 'Search for assets by the given parameters.',
    ),
    field(
      'tags',
      listOf(tagCountType.nonNullable()).nonNullable(),
      resolve: (context, args) {
        return [
          {'value': 'cat', 'count': 10}
        ];
      },
      description:
          'Retrieve the list of tags and their associated asset count.',
    ),
    field(
      'years',
      listOf(yearCountType.nonNullable()).nonNullable(),
      resolve: (context, args) {
        return [
          {'value': 2018, 'count': 146}
        ];
      },
      description:
          'Retrieve the list of years and their associated asset count.',
    ),
  ],
);

final GraphQLInputObjectType assetInputType = inputObjectType(
  'AssetInput',
  description: 'Definition for updating the information for an asset.',
  inputFields: [
    inputField(
      'tags',
      listOf(graphQLString.nonNullable()),
      description: 'New set of asset tags.',
    ),
    inputField(
      'caption',
      graphQLString,
      description: 'New asset caption.',
    ),
    inputField(
      'location',
      graphQLString,
      description: 'New asset location.',
    ),
    inputField(
      'datetime',
      graphQLDate,
      description: 'A date/time that overrides intrinsic values.',
    ),
    inputField(
      'mimetype',
      graphQLString,
      description: 'New asset media type.',
    ),
  ],
);

final GraphQLObjectType mutationType = objectType(
  'Mutation',
  description: 'Operations for modifying backend data.',
  fields: [
    field(
      'upload',
      graphQLId.nonNullable(),
      inputs: [
        GraphQLFieldInput(
          'file',
          graphQLUpload.nonNullable(),
        ),
      ],
      resolve: (context, args) {
        return null;
      },
      description: 'Upload an asset, returning its identifier.',
    ),
    field(
      'update',
      assetType.nonNullable(),
      inputs: [
        GraphQLFieldInput(
          'id',
          graphQLId.nonNullable(),
        ),
        GraphQLFieldInput(
          'asset',
          assetInputType.nonNullable(),
        ),
      ],
      resolve: (context, args) {
        return null;
      },
      description: 'Update the asset definition with the given values.',
    ),
  ],
);

final graphql = GraphQL(
  graphQLSchema(
    queryType: queryType,
    mutationType: mutationType,
  ),
);
