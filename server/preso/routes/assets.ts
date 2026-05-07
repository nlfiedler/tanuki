//
// Copyright (c) 2025 Nathan Fiedler
//
import fs from 'node:fs/promises';
import express from 'express';
import multer from 'multer';
import container from 'tanuki/server/container.ts';
import logger from 'tanuki/server/logger.ts';
import {
  MAX_RENDER_DIMENSION,
  parsePositiveInt,
  renderResizedJpeg
} from 'tanuki/server/preso/routes/render-helpers.ts';

const settings: any = container.resolve('settingsRepository');
const importAsset: any = container.resolve('importAsset');
// This router assumes that the blob repository implementation is going to be
// LocalBlobRepository, which generates URLs that map to the endpoints defined
// by this router. The coupling is bidirectional since the routes defined here
// will invoke methods on LocalBlobRepository to get the local asset path.
const blobs: any = container.resolve('blobRepository');
const records: any = container.resolve('recordRepository');

const uploads = settings.get('UPLOAD_PATH');
await fs.mkdir(uploads, { recursive: true });
const storage = multer.diskStorage({
  destination: function (req, file, cb) {
    cb(null, uploads);
  }
});
const upload = multer({ storage: storage });

const router = express.Router();

// Assets are always uploaded to this route regardless of which blob server is
// in use, since the import usecase needs to be able to process the raw file
// data in order to populate the fields of the database record.
router.post(
  '/upload',
  upload.single('content'),
  async function (req, res, _next) {
    // req.file is the incoming file object; req.body will hold any additional
    // fields provided in the request; the client is expected to provide the file
    // "last modified" date-time in Unix time as a string in 'last_modified'
    const modified =
      'last_modified' in req.body
        ? new Date(Number.parseInt(req.body.last_modified))
        : new Date();
    const file = req.file!;
    const asset = await importAsset(
      file.path,
      file.originalname,
      file.mimetype,
      modified
    );
    logger.info(`asset ${file.originalname} imported as ${asset.key}`);
    res.json({ ok: true, assetId: asset.key });
  }
);

router.get('/preview/:id', async function (req, res, _next) {
  const id = req.params.id;
  const widthRaw =
    typeof req.query.width === 'string' ? req.query.width : undefined;
  const heightRaw =
    typeof req.query.height === 'string' ? req.query.height : undefined;
  const hasWidth = widthRaw !== undefined && widthRaw.length > 0;
  const hasHeight = heightRaw !== undefined && heightRaw.length > 0;
  if (hasWidth === hasHeight) {
    res.status(400).send('exactly one of width or height is required');
    return;
  }
  const width = hasWidth ? parsePositiveInt(widthRaw) : undefined;
  const height = hasHeight ? parsePositiveInt(heightRaw) : undefined;
  if ((hasWidth && width === null) || (hasHeight && height === null)) {
    res
      .status(400)
      .send(
        `width/height must be a positive integer no greater than ${MAX_RENDER_DIMENSION}`
      );
    return;
  }
  try {
    const result = await renderResizedJpeg(blobs.blobPath(id), {
      width: width ?? undefined,
      height: height ?? undefined,
      withoutEnlargement: true
    });
    res.set({ 'Content-Type': 'image/jpeg' });
    res.send(result);
  } catch (error: any) {
    logger.error(error);
    res.redirect('/placeholder.svg');
  }
});

router.get('/thumbnail/:w/:h/:id', async function (req, res, _next) {
  const width = parsePositiveInt(req.params.w);
  const height = parsePositiveInt(req.params.h);
  const id = req.params.id;
  if (width === null || height === null) {
    res
      .status(400)
      .send(
        `width and height must be positive integers no greater than ${MAX_RENDER_DIMENSION}`
      );
    return;
  }
  try {
    const result = await renderResizedJpeg(blobs.blobPath(id), {
      width,
      height,
      fit: 'inside',
      withoutEnlargement: true
    });
    // thumbnails will always be in JPEG format
    res.set({ 'Content-Type': 'image/jpeg' });
    // send() handles Content-Length and ETag support
    res.send(result);
  } catch (error: any) {
    logger.error(error);
    res.redirect('/placeholder.svg');
  }
});

router.get('/raw/:id', async function (req, res, next) {
  const assetId = req.params.id;
  const filepath = blobs.blobPath(assetId);
  const asset = await records.getAssetById(assetId);
  const mimetype = asset.mediaType ?? 'application/octet-stream';
  // sendFile() handles Content-Length, ETag, and Range requests
  res.sendFile(
    filepath,
    { headers: { 'Content-Type': mimetype } },
    function (err) {
      // only call next() if there is an error value
      if (err) {
        next(err);
      }
    }
  );
});

export default router;
