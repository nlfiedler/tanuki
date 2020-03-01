//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:io';
import 'package:angel_container/angel_container.dart';
import 'package:angel_cors/angel_cors.dart';
import 'package:angel_framework/angel_framework.dart';
import 'package:angel_framework/http.dart';
import 'package:angel_graphql/angel_graphql.dart';
import 'package:angel_static/angel_static.dart';
import 'package:file/local.dart';
import 'package:logging/logging.dart';
import 'package:pretty_logging/pretty_logging.dart';
import 'package:tanuki/schema.dart' as schema;

main() async {
  Logger.root.onRecord.listen(prettyLog);
  var app = Angel(
    logger: Logger('angel'),
    reflector: EmptyReflector(),
  );

  app.all('/graphql', graphQLHttp(schema.graphql), middleware: [cors()]);
  // presently the schema parsing fails, but can still perform queries
  // (c.f. https://github.com/angel-dart/angel/issues/241)
  app.get('/graphiql', graphiQL());

  // serve static files and use index.html as the fallback
  var fs = const LocalFileSystem();
  var vDir = CachingVirtualDirectory(app, fs);
  app.fallback(vDir.handleRequest);
  app.fallback(vDir.pushState('index.html'));

  // start the server on the designated host and port
  var http = AngelHttp(app);
  var host = Platform.environment['HOST'];
  var portVar = Platform.environment['PORT'];
  var port = portVar == null ? 4040 : int.parse(portVar);
  await http.startServer(host ?? 'localhost', port);
  print('listening at ${http.uri}');
}
