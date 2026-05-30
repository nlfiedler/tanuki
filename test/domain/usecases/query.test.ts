//
// Copyright (c) 2025 Nathan Fiedler
//
import { describe, expect, test } from 'bun:test';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { Location } from 'tanuki/server/domain/entities/location.ts';
import { SyntheticData } from 'tanuki/server/domain/entities/synthetic-data.ts';
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

    test('should parse query and match by ML label', async function () {
      const synthetic = new SyntheticData();
      synthetic.labels = ['beach', 'palm tree', 'sunset'];
      synthetic.primaryLabel = 'beach';
      const labelled = new Asset('monday1')
        .setChecksum('sha1-cafebabe')
        .setFilename('img_1234.jpg')
        .setByteLength(1024)
        .setMediaType('image/jpeg')
        .setTags(['vacation'])
        .setImportDate(new Date(2018, 4, 31, 21, 10, 11))
        .setSynthetic(synthetic);
      const unlabelled = new Asset('monday2')
        .setChecksum('sha1-deadbeef')
        .setFilename('img_5678.jpg')
        .setByteLength(1024)
        .setMediaType('image/jpeg')
        .setTags(['vacation'])
        .setImportDate(new Date(2018, 4, 31, 21, 10, 11));

      // primary label matches
      const primary = await parse('label:beach');
      expect(primary.matches(labelled)).toBeTrue();
      // secondary labels also match (case-insensitive)
      const secondary = await parse('label:"palm tree"');
      expect(secondary.matches(labelled)).toBeTrue();
      const upper = await parse('label:SUNSET');
      expect(upper.matches(labelled)).toBeTrue();
      // non-matching label
      const miss = await parse('label:mountain');
      expect(miss.matches(labelled)).toBeFalse();
      // assets without synthetic data never match
      expect(primary.matches(unlabelled)).toBeFalse();
      // composes with other predicates via and/or/-
      const compound = await parse('label:beach and tag:vacation');
      expect(compound.matches(labelled)).toBeTrue();
      const negated = await parse('-label:mountain');
      expect(negated.matches(labelled)).toBeTrue();
    });

    test('should parse and match by person via the resolver', async function () {
      const alice = new Asset('alice1')
        .setChecksum('sha1-aaa')
        .setFilename('a.jpg')
        .setByteLength(1024)
        .setMediaType('image/jpeg')
        .setTags(['party'])
        .setImportDate(new Date(2018, 4, 31, 21, 10, 11));
      const other = new Asset('other1')
        .setChecksum('sha1-bbb')
        .setFilename('b.jpg')
        .setByteLength(1024)
        .setMediaType('image/jpeg')
        .setTags(['party'])
        .setImportDate(new Date(2018, 4, 31, 21, 10, 11));

      const cons = await parse('person:p1', resolvePerson);
      expect(cons.matches(alice)).toBeTrue();
      expect(cons.matches(other)).toBeFalse();

      // composes with other predicates
      const compound = await parse('person:p1 and tag:party', resolvePerson);
      expect(compound.matches(alice)).toBeTrue();
      expect(compound.matches(other)).toBeFalse();

      // without a resolver the predicate is unsupported
      expect(parse('person:p1')).rejects.toThrow('not supported');
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

    test('should parse query and match by has', async function () {
      const bare = new Asset('monday1')
        .setChecksum('sha1-cafebabe')
        .setFilename('img_1234.jpg')
        .setByteLength(1024)
        .setMediaType('image/jpeg')
        .setImportDate(new Date(2018, 4, 31, 21, 10, 11));

      const has_caption = await parse('has:caption');
      expect(has_caption.matches(bare)).toBeFalse();
      const has_location = await parse('has:location');
      expect(has_location.matches(bare)).toBeFalse();
      const has_user_date = await parse('has:userDate');
      expect(has_user_date.matches(bare)).toBeFalse();
      const has_tags = await parse('has:tags');
      expect(has_tags.matches(bare)).toBeFalse();
      const has_filename = await parse('has:filename');
      expect(has_filename.matches(bare)).toBeTrue();
      const has_unknown = await parse('has:nonexistent');
      expect(has_unknown.matches(bare)).toBeFalse();

      const populated = new Asset('monday2')
        .setChecksum('sha1-cafebabe')
        .setFilename('img_1234.jpg')
        .setByteLength(1024)
        .setMediaType('image/jpeg')
        .setTags(['kitten', 'puppy'])
        .setImportDate(new Date(2018, 4, 31, 21, 10, 11))
        .setCaption('a fine afternoon')
        .setLocation(Location.parse('museum; Paris, France'))
        .setUserDate(new Date(2018, 5, 1, 12, 0, 0))
        .setOriginalDate(new Date(2018, 4, 31, 18, 30, 0));

      const has_caption_pop = await parse('has:caption');
      expect(has_caption_pop.matches(populated)).toBeTrue();
      const has_location_pop = await parse('has:location');
      expect(has_location_pop.matches(populated)).toBeTrue();
      const has_tags_pop = await parse('has:tags');
      expect(has_tags_pop.matches(populated)).toBeTrue();
      const has_user_camel = await parse('has:userDate');
      expect(has_user_camel.matches(populated)).toBeTrue();
      const has_user_snake = await parse('has:user_date');
      expect(has_user_snake.matches(populated)).toBeTrue();
      const has_user_kebab = await parse('has:user-date');
      expect(has_user_kebab.matches(populated)).toBeTrue();
      const has_user_upper = await parse('has:USERDATE');
      expect(has_user_upper.matches(populated)).toBeTrue();
      const has_original_snake = await parse('has:original_date');
      expect(has_original_snake.matches(populated)).toBeTrue();

      const combined = await parse('has:caption and tag:kitten');
      expect(combined.matches(populated)).toBeTrue();
      expect(combined.matches(bare)).toBeFalse();
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

/** Test resolver mapping person id `p1` to a single asset. */
async function resolvePerson(id: string): Promise<Set<string>> {
  return id === 'p1' ? new Set(['alice1']) : new Set<string>();
}

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
