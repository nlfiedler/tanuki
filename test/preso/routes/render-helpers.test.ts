//
// Copyright (c) 2026 Nathan Fiedler
//
import { describe, expect, test } from 'bun:test';
import sharp from 'sharp';
// prepare the test environment as early as possible
import 'tanuki/test/env.ts';
import {
  MAX_RENDER_DIMENSION,
  isVideoPath,
  parsePositiveInt,
  renderResizedJpeg
} from 'tanuki/server/preso/routes/render-helpers.ts';

describe('parsePositiveInt', function () {
  test('accepts a normal positive integer', function () {
    expect(parsePositiveInt('480')).toBe(480);
  });

  test('accepts the upper bound', function () {
    expect(parsePositiveInt(String(MAX_RENDER_DIMENSION))).toBe(
      MAX_RENDER_DIMENSION
    );
  });

  test('rejects values above the upper bound', function () {
    expect(parsePositiveInt(String(MAX_RENDER_DIMENSION + 1))).toBeNull();
    expect(parsePositiveInt('100000')).toBeNull();
  });

  test('rejects zero', function () {
    expect(parsePositiveInt('0')).toBeNull();
  });

  test('rejects negative numbers', function () {
    // negatives contain a non-digit character so they fail the regex
    expect(parsePositiveInt('-5')).toBeNull();
  });

  test('rejects non-numeric strings', function () {
    expect(parsePositiveInt('foo')).toBeNull();
    expect(parsePositiveInt('')).toBeNull();
  });

  test('rejects values with trailing junk', function () {
    // Number.parseInt would otherwise return 5 here, masking bad input
    expect(parsePositiveInt('5abc')).toBeNull();
  });

  test('rejects undefined', function () {
    // eslint-disable-next-line unicorn/no-useless-undefined
    expect(parsePositiveInt(undefined)).toBeNull();
  });

  test('rejects fractional values', function () {
    expect(parsePositiveInt('5.5')).toBeNull();
  });

  test('honors a custom maximum', function () {
    expect(parsePositiveInt('100', 50)).toBeNull();
    expect(parsePositiveInt('50', 50)).toBe(50);
  });
});

describe('isVideoPath', function () {
  test('matches common video extensions', function () {
    expect(isVideoPath('foo.mp4')).toBeTrue();
    expect(isVideoPath('foo.MOV')).toBeTrue();
    expect(isVideoPath('a/b/c.mkv')).toBeTrue();
    expect(isVideoPath('clip.WebM')).toBeTrue();
  });

  test('does not match image or other extensions', function () {
    expect(isVideoPath('foo.jpg')).toBeFalse();
    expect(isVideoPath('foo.png')).toBeFalse();
    expect(isVideoPath('foo.pdf')).toBeFalse();
    expect(isVideoPath('foo')).toBeFalse();
  });
});

describe('renderResizedJpeg', function () {
  test('resizes an image into a width-bounded JPEG', async function () {
    const out = await renderResizedJpeg(
      './test/fixtures/fighting_kittens.jpg',
      { width: 200, withoutEnlargement: true }
    );
    const meta = await sharp(out).metadata();
    expect(meta.format).toBe('jpeg');
    expect(meta.width).toBe(200);
  });

  test('extracts a frame from a video and resizes it', async function () {
    const out = await renderResizedJpeg('./test/fixtures/ooo_tracks.mp4', {
      width: 160,
      withoutEnlargement: true
    });
    const meta = await sharp(out).metadata();
    expect(meta.format).toBe('jpeg');
    expect(meta.width).toBe(160);
  });
});
