//
// Copyright (c) 2020 Nathan Fiedler
//

/// Thrown when remote server error occurs.
class ServerException implements Exception {
  final String message;

  const ServerException([this.message]);

  @override
  String toString() => message ?? 'ServerException';
}
