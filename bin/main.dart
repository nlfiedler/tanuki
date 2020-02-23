//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:io';
import 'package:http_parser/http_parser.dart';
import 'package:path/path.dart' as p;
import 'package:shelf/shelf.dart' as shelf;
import 'package:shelf/shelf_io.dart' as io;
import 'package:shelf_router/shelf_router.dart';
import 'package:shelf_static/shelf_static.dart';

main() async {
  var app = Router();

  app.post('/graphql', (shelf.Request request) {
    // canned graphql response for a "tags" query
    var json = '''{
      "data": {
        "tags": [
          { "value": "dog", "count": 6 },
          { "value": "cat", "count": 7 },
          { "value": "bird", "count": 4 },
          { "value": "mouse", "count": 8 }
        ]
      }
    }''';
    return shelf.Response.ok(json);
  });

  var staticPath = Platform.environment['STATIC_PATH'] ?? 'public';
  app.all('/<ignored|.*>', (shelf.Request request) {
    // the essence of VirtualDirectory to serve a single file
    var file = File(p.join(staticPath, 'index.html'));
    var stat = file.statSync();
    var ifModifiedSince = request.ifModifiedSince;
    if (ifModifiedSince != null) {
      var fileChangeAtSecResolution = toSecondResolution(stat.changed);
      if (!fileChangeAtSecResolution.isAfter(ifModifiedSince)) {
        return shelf.Response.notModified();
      }
    }
    var headers = {
      HttpHeaders.contentLengthHeader: stat.size.toString(),
      HttpHeaders.lastModifiedHeader: formatHttpDate(stat.changed),
      HttpHeaders.contentTypeHeader: 'text/html'
    };
    return shelf.Response.ok(file.openRead(), headers: headers);
  });

  // The static file handler will attempt to find matching files. When that
  // fails, then the router takes over and handles everything else, including
  // the fallback of serving the index page. In this scenario, the front-end is
  // responsible for dealing with the application routing.
  var cascade = shelf.Cascade()
      .add(createStaticHandler(staticPath, defaultDocument: 'index.html'))
      .add(app.handler);
  var handler = const shelf.Pipeline()
      .addMiddleware(shelf.logRequests())
      .addMiddleware(shelf.createMiddleware(responseHandler: addCorsHeaders))
      .addHandler(cascade.handler);

  var host = Platform.environment['HOST'];
  var portVar = Platform.environment['PORT'];
  var port = portVar == null ? 4040 : int.parse(portVar);
  var server = await io.serve(handler, host ?? 'localhost', port);
  server.autoCompress = true;

  print('listening on http://${server.address.host}:${server.port}');
}

shelf.Response addCorsHeaders(shelf.Response response) {
  Map<String, String> headers = Map.from(response.headers);
  headers['Access-Control-Allow-Origin'] = '*';
  headers['Access-Control-Allow-Methods'] = 'POST, GET, OPTIONS';
  headers['Access-Control-Allow-Headers'] = 'Authorization';
  headers['Access-Control-Max-Age'] = '86400';
  return response.change(headers: headers);
}

DateTime toSecondResolution(DateTime dt) {
  if (dt.millisecond == 0) return dt;
  return dt.subtract(Duration(milliseconds: dt.millisecond));
}
