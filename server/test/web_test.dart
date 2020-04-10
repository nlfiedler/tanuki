//
// Copyright (c) 2020 Nathan Fiedler
//
import 'package:angel_framework/angel_framework.dart';
import 'package:angel_test/angel_test.dart';
import 'package:http/http.dart' as http;
import 'package:test/test.dart';
import '../bin/main.dart' as web;

void main() {
  Angel app;
  TestClient client;

  setUp(() async {
    app = Angel();
    web.addRoutes(app);
    client = await connectTo(app);
  });

  tearDown(() async {
    await client.close();
  });

  test('cors probe returns expected headers', () async {
    var req = http.Request('OPTIONS', Uri(path: '/graphql'));
    var res = await client.send(req);
    expect(res.statusCode, 204);
    expect(res.headers['access-control-allow-origin'], '*');
    expect(res.headers['access-control-allow-methods'],
        'GET,HEAD,PUT,PATCH,POST,DELETE');
    expect(res.headers['content-length'], '0');
    expect(res.headers['content-type'], 'text/plain');
  });

  test('static image returns correct media type', () async {
    var res = await client.get('/icons/Icon-192.png');
    expect(res, hasStatus(200));
    expect(res, hasHeader('content-length', 5292));
    expect(res, hasContentType('image/png'));
    expect(res, hasBody());
  });

  test('fallback route returns index.html', () async {
    var res = await client.get('/foobar');
    expect(res, hasStatus(200));
    expect(res, hasContentType('text/html'));
    expect(res, hasBody());
  });
}
