//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:convert';
import 'dart:io';
import 'package:angel_container/angel_container.dart';
import 'package:angel_cors/angel_cors.dart';
import 'package:angel_framework/angel_framework.dart';
import 'package:angel_framework/http.dart';
import 'package:angel_static/angel_static.dart';
import 'package:file/local.dart';
import 'package:http_parser/http_parser.dart';
import 'package:logging/logging.dart';
import 'package:pretty_logging/pretty_logging.dart';

main() async {
  Logger.root.onRecord.listen(prettyLog);
  var app = Angel(
    logger: Logger('angel'),
    reflector: EmptyReflector(),
  );
  app.all(
    '/graphql',
    chain(
      [
        cors(),
        (req, res) {
          // canned graphql response for a "tags" query
          var data = {
            'data': {
              'tags': [
                {'value': 'dog', 'count': 6},
                {'value': 'cat', 'count': 7},
                {'value': 'bird', 'count': 4},
                {'value': 'mouse', 'count': 8},
              ]
            }
          };
          res.contentType =
              MediaType('application', 'json', {'charset': 'utf-8'});
          var text = json.encode(data);
          res.contentLength = text.length; // assumes UTF-8 text
          res.write(text);
        }
      ],
    ),
  );

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
