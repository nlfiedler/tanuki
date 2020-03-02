//
// Copyright (c) 2020 Nathan Fiedler
//
import 'dart:io';
import 'package:exifdart/exifdart_io.dart';
import 'package:image/image.dart';
import 'package:mime/mime.dart';
import 'package:test/test.dart';

// The image package lacks full EXIF support for the time being (c.f.
// https://github.com/brendan-duncan/image/issues/132)
//
// So for now use the exifdart package, which returns string keys.
const dateTimeOriginal = 'DateTimeOriginal';
const orientation = 'Orientation';

void main() {
  test('Read jpeg data using image package', () {
    final path = 'test/fixtures/fighting_kittens.jpg';
    Image image = decodeImage(File(path).readAsBytesSync());
    // image is rotated to width/height are reversed
    expect(image.width, equals(384));
    expect(image.height, equals(512));
    expect(image.exif.hasOrientation, equals(true));
    expect(image.exif.orientation, equals(8));
  });

  test('Read exif data using exifdart package', () async {
    final path = 'test/fixtures/dcp_1069.jpg';
    Map<String, dynamic> data = await readExifFromFile(File(path));
    expect(data, isNotEmpty);
    expect(data[dateTimeOriginal], equals('2003:09:03 17:24:35'));
    expect(data[orientation], equals(1));
  });

  // resize an image using copyResize(Image)
  // rotate an image using copyRotate(Image)

  test('detect media type of files', () {
    expect(lookupMimeType('test.html'), equals('text/html'));
    expect(lookupMimeType('test', headerBytes: [0xFF, 0xD8]),
        equals('image/jpeg'));
    // mime package is more accurate with header bytes
    expect(lookupMimeType('test.html', headerBytes: [0xFF, 0xD8]),
        equals('image/jpeg'));
  });
}
