//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, test } from 'bun:test';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { Location } from 'tanuki/server/domain/entities/location.ts';
import { AsyncQueue } from 'tanuki/server/shared/collections/async-queue.ts';
import {
  lex,
  TokenType,
  Token,
  parse
} from 'tanuki/server/domain/usecases/query.ts';

describe('Query language support', function () {
  describe('Query lexer', function () {
    test('should lex an empty string and return Eof', async function () {
      const chan = lex('');
      const token = await chan.dequeue();
      expect(token.typ).toEqual(TokenType.Eof);
    });

    test('should lex whitespace and return Eof', async function () {
      const chan = lex('   \r  \n   \t  ');
      const token = await chan.dequeue();
      expect(token.typ).toEqual(TokenType.Eof);
    });

    test('should grouping operators and ignore whitespace', async function () {
      const chan = lex('     (\n\t )\r\n');
      const tokens = await drainTokens(chan);
      expect(tokens).toHaveLength(3);
      expect(tokens[0]?.typ).toEqual(TokenType.Open);
      expect(tokens[1]?.typ).toEqual(TokenType.Close);
      expect(tokens[2]?.typ).toEqual(TokenType.Eof);
    });

    test('should lex predicates with simple arguments', async function () {
      const chan = lex(
        'tag:kittens -tag:clouds loc:\'castro valley\' loc:"lower manhatten"'
      );
      const actual = await drainTokens(chan);
      expect(actual).toHaveLength(14);
      const expected = [
        new Token(TokenType.Predicate, 'tag'),
        new Token(TokenType.Colon, ':'),
        new Token(TokenType.Arg, 'kittens'),
        new Token(TokenType.Not, '-'),
        new Token(TokenType.Predicate, 'tag'),
        new Token(TokenType.Colon, ':'),
        new Token(TokenType.Arg, 'clouds'),
        new Token(TokenType.Predicate, 'loc'),
        new Token(TokenType.Colon, ':'),
        new Token(TokenType.Arg, 'castro valley'),
        new Token(TokenType.Predicate, 'loc'),
        new Token(TokenType.Colon, ':'),
        new Token(TokenType.Arg, 'lower manhatten'),
        new Token(TokenType.Eof, '')
      ];
      for (const [i, element] of actual.entries()) {
        expect(element).toEqual(expected[i]!);
      }
    });

    test('should lex predicates with complex arguments', async function () {
      const chan = lex('loc:city:london or loc:region:japan loc:label:');
      const actual = await drainTokens(chan);
      expect(actual).toHaveLength(16);
      const expected = [
        new Token(TokenType.Predicate, 'loc'),
        new Token(TokenType.Colon, ':'),
        new Token(TokenType.Arg, 'city'),
        new Token(TokenType.Colon, ':'),
        new Token(TokenType.Arg, 'london'),
        new Token(TokenType.Or, 'or'),
        new Token(TokenType.Predicate, 'loc'),
        new Token(TokenType.Colon, ':'),
        new Token(TokenType.Arg, 'region'),
        new Token(TokenType.Colon, ':'),
        new Token(TokenType.Arg, 'japan'),
        new Token(TokenType.Predicate, 'loc'),
        new Token(TokenType.Colon, ':'),
        new Token(TokenType.Arg, 'label'),
        new Token(TokenType.Colon, ':'),
        new Token(TokenType.Eof, '')
      ];
      for (const [i, element] of actual.entries()) {
        expect(element).toEqual(expected[i]!);
      }
    });

    test('should lex basic and/or operators', async function () {
      const chan = lex('tag:kittens or tag:clouds and tag:rain');
      const actual = await drainTokens(chan);
      expect(actual).toHaveLength(12);
      const expected = [
        new Token(TokenType.Predicate, 'tag'),
        new Token(TokenType.Colon, ':'),
        new Token(TokenType.Arg, 'kittens'),
        new Token(TokenType.Or, 'or'),
        new Token(TokenType.Predicate, 'tag'),
        new Token(TokenType.Colon, ':'),
        new Token(TokenType.Arg, 'clouds'),
        new Token(TokenType.And, 'and'),
        new Token(TokenType.Predicate, 'tag'),
        new Token(TokenType.Colon, ':'),
        new Token(TokenType.Arg, 'rain'),
        new Token(TokenType.Eof, '')
      ];
      for (const [i, element] of actual.entries()) {
        expect(element).toEqual(expected[i]!);
      }
    });

    test('should lex repeated negation', async function () {
      const chan = lex('--tag:kittens or - tag:clouds');
      const actual = await drainTokens(chan);
      expect(actual).toHaveLength(11);
      const expected = [
        new Token(TokenType.Not, '-'),
        new Token(TokenType.Not, '-'),
        new Token(TokenType.Predicate, 'tag'),
        new Token(TokenType.Colon, ':'),
        new Token(TokenType.Arg, 'kittens'),
        new Token(TokenType.Or, 'or'),
        new Token(TokenType.Not, '-'),
        new Token(TokenType.Predicate, 'tag'),
        new Token(TokenType.Colon, ':'),
        new Token(TokenType.Arg, 'clouds'),
        new Token(TokenType.Eof, '')
      ];
      for (const [i, element] of actual.entries()) {
        expect(element).toEqual(expected[i]!);
      }
    });

    test('should lex example from perkeep web site', async function () {
      const chan = lex(
        '-(after:"2010-01-01" before:"2010-03-02T12:33:44") or loc:"Amsterdam"'
      );
      const actual = await drainTokens(chan);
      expect(actual).toHaveLength(14);
      const expected = [
        new Token(TokenType.Not, '-'),
        new Token(TokenType.Open, '('),
        new Token(TokenType.Predicate, 'after'),
        new Token(TokenType.Colon, ':'),
        new Token(TokenType.Arg, '2010-01-01'),
        new Token(TokenType.Predicate, 'before'),
        new Token(TokenType.Colon, ':'),
        new Token(TokenType.Arg, '2010-03-02T12:33:44'),
        new Token(TokenType.Close, ')'),
        new Token(TokenType.Or, 'or'),
        new Token(TokenType.Predicate, 'loc'),
        new Token(TokenType.Colon, ':'),
        new Token(TokenType.Arg, 'Amsterdam'),
        new Token(TokenType.Eof, '')
      ];
      for (const [i, element] of actual.entries()) {
        expect(element).toEqual(expected[i]!);
      }
    });
  });

  describe('Query parser', function () {
    test('should parse query and match by tag', async function () {
      const asset = new Asset('monday1')
        .setChecksum('sha1-cafebabe')
        .setFilename('img_1234.jpg')
        .setByteLength(1024)
        .setMediaType('image/jpeg')
        .setTags(['kitten', 'puppy'])
        .setImportDate(new Date(2018, 4, 31, 21, 10, 11));

      const kitten = await parse('tag:kitten');
      expect(kitten.matches(asset)).toBeTrue();

      const both = await parse('tag:kitten tag:puppy');
      expect(both.matches(asset)).toBeTrue();

      const either = await parse('tag:kitten or tag:fluffy');
      expect(either.matches(asset)).toBeTrue();

      const notfluffy = await parse('-tag:fluffy');
      expect(notfluffy.matches(asset)).toBeTrue();

      const notkitten = await parse('-tag:kitten');
      expect(notkitten.matches(asset)).toBeFalse();

      const andfail = await parse('tag:kitten tag:fluffy');
      expect(andfail.matches(asset)).toBeFalse();

      const fluffy = await parse('tag:fluffy');
      expect(fluffy.matches(asset)).toBeFalse();

      const neither = await parse('tag:fluffy or tag:furry');
      expect(neither.matches(asset)).toBeFalse();
    });

    test('should parse query and match by type', async function () {
      const asset = new Asset('monday1')
        .setChecksum('sha1-cafebabe')
        .setFilename('img_1234.jpg')
        .setByteLength(1024)
        .setMediaType('image/jpeg')
        .setTags(['kitten', 'puppy'])
        .setImportDate(new Date(2018, 4, 31, 21, 10, 11));

      const image = await parse('is:image');
      expect(image.matches(asset)).toBeTrue();

      const video = await parse('is:video');
      expect(video.matches(asset)).toBeFalse();
    });

    test('should parse query and match by subtype', async function () {
      const asset = new Asset('monday1')
        .setChecksum('sha1-cafebabe')
        .setFilename('img_1234.jpg')
        .setByteLength(1024)
        .setMediaType('image/jpeg')
        .setTags(['kitten', 'puppy'])
        .setImportDate(new Date(2018, 4, 31, 21, 10, 11));

      const jpeg = await parse('format:jpeg');
      expect(jpeg.matches(asset)).toBeTrue();

      const png = await parse('format:png');
      expect(png.matches(asset)).toBeFalse();
    });

    test('should parse query and match by filename', async function () {
      const asset = new Asset('monday1')
        .setChecksum('sha1-cafebabe')
        .setFilename('img_1234.jpg')
        .setByteLength(1024)
        .setMediaType('image/jpeg')
        .setTags(['kitten', 'puppy'])
        .setImportDate(new Date(2018, 4, 31, 21, 10, 11));

      const good = await parse('filename:img_1234.jpg');
      expect(good.matches(asset)).toBeTrue();

      const bad = await parse('filename:IMG_4321.JPG');
      expect(bad.matches(asset)).toBeFalse();
    });

    test('should parse query and match by location', async function () {
      const asset = new Asset('monday1')
        .setChecksum('sha1-cafebabe')
        .setFilename('img_1234.jpg')
        .setByteLength(1024)
        .setMediaType('image/jpeg')
        .setTags(['kitten', 'puppy'])
        .setImportDate(new Date(2018, 4, 31, 21, 10, 11))
        .setLocation(Location.parse('museum; Paris, France'));

      const paris = await parse('loc:paris');
      expect(paris.matches(asset)).toBeTrue();

      const france = await parse('loc:france');
      expect(france.matches(asset)).toBeTrue();

      const museum_label = await parse('loc:label:museum');
      expect(museum_label.matches(asset)).toBeTrue();

      const museum_any = await parse('loc:any:museum');
      expect(museum_any.matches(asset)).toBeTrue();

      const paris_city = await parse('loc:city:paris');
      expect(paris_city.matches(asset)).toBeTrue();

      const france_region = await parse('loc:region:france');
      expect(france_region.matches(asset)).toBeTrue();

      const paris_france = await parse('loc:city:paris loc:region:france');
      expect(paris_france.matches(asset)).toBeTrue();

      const beach_or_paris = await parse('loc:beach or loc:city:paris');
      expect(beach_or_paris.matches(asset)).toBeTrue();

      const beach = await parse('loc:beach');
      expect(beach.matches(asset)).toBeFalse();
    });

    test('should parse query and match empty location', async function () {
      const asset = new Asset('monday1')
        .setChecksum('sha1-cafebabe')
        .setFilename('img_1234.jpg')
        .setByteLength(1024)
        .setMediaType('image/jpeg')
        .setTags(['kitten', 'puppy'])
        .setImportDate(new Date(2018, 4, 31, 21, 10, 11))
        .setLocation(Location.parse('Paris, France'));

      const no_label = await parse('loc:label:');
      expect(no_label.matches(asset)).toBeTrue();

      const any_blank = await parse('loc:any:');
      expect(any_blank.matches(asset)).toBeTrue();

      const no_label_and_paris = await parse('loc:label: loc:city:paris');
      expect(no_label_and_paris.matches(asset)).toBeTrue();
    });

    test('should parse groups and match accordingly', async function () {
      const asset = new Asset('monday1')
        .setChecksum('sha1-cafebabe')
        .setFilename('img_1234.jpg')
        .setByteLength(1024)
        .setMediaType('image/jpeg')
        .setTags(['kitten', 'puppy'])
        .setImportDate(new Date(2018, 4, 31, 21, 10, 11));

      const eitheror = await parse('(tag:kitten or tag:fluffy) and is:image');
      expect(eitheror.matches(asset)).toBeTrue();

      const eitherand = await parse('tag:kitten or (tag:fluffy and is:image)');
      expect(eitherand.matches(asset)).toBeTrue();
    });
  });
});

async function drainTokens(chan: AsyncQueue<Token>): Promise<Array<Token>> {
  const tokens = [];
  while (true) {
    const token = await chan.dequeue();
    tokens.push(token);
    if (token.typ == TokenType.Eof) {
      break;
    }
  }
  return tokens;
}
