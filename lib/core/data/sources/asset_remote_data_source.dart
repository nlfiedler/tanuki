//
// Copyright (c) 2024 Nathan Fiedler
//
import 'dart:convert';
import 'dart:typed_data';
import 'package:graphql/client.dart';
import 'package:http/http.dart' as http;
import 'package:http_parser/http_parser.dart' as parser;
import 'package:mime/mime.dart';
import 'package:path/path.dart' as p;
import 'package:tanuki/core/error/exceptions.dart' as err;

abstract class AssetRemoteDataSource {
  /// Import all of the assets in the 'uploads' directory.
  Future<int> ingestAssets();

  /// Upload a file to replace the content of an existing asset.
  Future<String> replaceAssetBytes(
      String assetId, String filename, Uint8List contents);

  /// Upload the given asset to the asset store.
  Future<String> uploadAsset(String filepath);

  /// Upload a file with the given name and contents to the asset store.
  Future<String> uploadAssetBytes(String filename, Uint8List contents);
}

class AssetRemoteDataSourceImpl extends AssetRemoteDataSource {
  final http.Client httpClient;
  final GraphQLClient gqlClient;
  final String baseUrl;

  AssetRemoteDataSourceImpl({
    required this.httpClient,
    required this.baseUrl,
    required this.gqlClient,
  });

  @override
  Future<int> ingestAssets() async {
    const getStore = r'''
      mutation {
        ingest
      }
    ''';
    final mutationOptions = MutationOptions(
      document: gql(getStore),
    );
    final QueryResult result = await gqlClient.mutate(mutationOptions);
    if (result.hasException) {
      throw err.ServerException(result.exception.toString());
    }
    return (result.data?['ingest'] ?? 0) as int;
  }

  @override
  Future<String> replaceAssetBytes(
      String assetId, String filename, Uint8List contents) async {
    // build up a multipart request based on the given information
    final uri = Uri.parse('$baseUrl/api/replace/$assetId');
    final request = http.MultipartRequest('POST', uri);
    final mimeType = lookupMimeType(filename) ?? 'application/octet-stream';
    final mimeParts = mimeType.split('/');
    final mediaType = parser.MediaType(mimeParts[0], mimeParts[1]);
    final multiFile = http.MultipartFile.fromBytes('asset', contents,
        filename: filename, contentType: mediaType);
    request.files.add(multiFile);
    return _performUpload(request);
  }

  @override
  Future<String> uploadAsset(String filepath) async {
    // build up a multipart request based on the given file
    final uri = Uri.parse('$baseUrl/api/import');
    final request = http.MultipartRequest('POST', uri);
    final filename = p.basename(filepath);
    final mimeType = lookupMimeType(filename) ?? 'application/octet-stream';
    final mimeParts = mimeType.split('/');
    final mediaType = parser.MediaType(mimeParts[0], mimeParts[1]);
    final multiFile = await http.MultipartFile.fromPath('asset', filepath,
        filename: filename, contentType: mediaType);
    request.files.add(multiFile);
    return _performUpload(request);
  }

  @override
  Future<String> uploadAssetBytes(String filename, Uint8List contents) async {
    // build up a multipart request based on the given information
    final uri = Uri.parse('$baseUrl/api/import');
    final request = http.MultipartRequest('POST', uri);
    final mimeType = lookupMimeType(filename) ?? 'application/octet-stream';
    final mimeParts = mimeType.split('/');
    final mediaType = parser.MediaType(mimeParts[0], mimeParts[1]);
    final multiFile = http.MultipartFile.fromBytes('asset', contents,
        filename: filename, contentType: mediaType);
    request.files.add(multiFile);
    return _performUpload(request);
  }

  Future<String> _performUpload(http.MultipartRequest request) async {
    // send the request using our http client instance
    final resp = await httpClient.send(request);
    if (resp.statusCode != 200) {
      throw err.ServerException('unexpected response: ${resp.statusCode}');
    }
    // collect asset identifier(s) from the JSON response
    final assetIds = <String>[];
    await for (String s in resp.stream.transform(utf8.decoder)) {
      List<dynamic> data = json.decode(s);
      assetIds.addAll(data.map((e) => e.toString()));
    }
    if (assetIds.length != 1) {
      throw err.ServerException(
          'received wrong number of identifiers: ${assetIds.length}');
    }
    return assetIds[0];
  }
}
