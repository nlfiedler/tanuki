//
// Copyright (c) 2025 Nathan Fiedler
//
import fs from 'node:fs/promises';
import express from 'express';
import multer from 'multer';
import sharp from 'sharp';
import container from 'tanuki/server/container.ts';
import logger from 'tanuki/server/logger.ts';

const settings = container.resolve('settingsRepository');
const importAsset = container.resolve('importAsset');
// This router assumes that the blob repository implementation is going to be
// LocalBlobRepository, which generates URLs that map to the endpoints defined
// by this router. The coupling is bidirectional since the routes defined here
// will invoke methods on LocalBlobRepository to get the local asset path.
const blobs = container.resolve('blobRepository');
const records = container.resolve('recordRepository');

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

router.get('/thumbnail/:w/:h/:id', async function (req, res, _next) {
  const width = Number.parseInt(req.params.w);
  const height = Number.parseInt(req.params.h);
  const id = req.params.id;
  try {
    // fit the image into a box of the given size, convert to jpeg
    const result = await sharp(blobs.blobPath(id), { autoOrient: true })
      .resize({
        width,
        height,
        fit: 'inside',
        withoutEnlargement: true
      })
      .toFormat('jpeg')
      .toBuffer();
    if (result === null) {
      res.status(404).send('no such asset');
    } else {
      // thumbnails will always be in JPEG format
      res.set({ 'Content-Type': 'image/jpeg' });
      // send() handles Content-Length and ETag support
      res.send(result);
    }
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
