//
// Copyright (c) 2025 Nathan Fiedler
//
import express from 'express';
import multer from 'multer';
import container from 'tanuki/server/container.ts';
import logger from 'tanuki/server/logger.ts';

const settings = container.resolve('settingsRepository');
const importAsset = container.resolve('importAsset');
const blobs = container.resolve('blobRepository');
const records = container.resolve('recordRepository');

const uploads = settings.get('UPLOAD_PATH');
const storage = multer.diskStorage({
  destination: function (req, file, cb) {
    cb(null, uploads);
  }
});
const upload = multer({ storage: storage });

const router = express.Router();

router.post('/upload', upload.single('file_blob'), async function (req, res, _next) {
  // req.file is the `file_blob` file object; req.body will hold any additional
  // fields provided in the request; the client is expected to provide the file
  // "last modified" date-time in Unix time as a string in 'last_modified'
  const modified = 'last_modified' in req.body ? new Date(parseInt(req.body.last_modified)) : new Date();
  const file = req.file!;
  const asset = await importAsset(file.path, file.originalname, file.mimetype, modified);
  logger.info(`asset ${file.originalname} imported as ${asset.key}`);
  res.redirect('/');
});

router.get('/thumbnail/:w/:h/:id', async function (req, res, _next) {
  const width = parseInt(req.params.w);
  const height = parseInt(req.params.h);
  const id = req.params.id;
  try {
    const result = await blobs.thumbnail(id, width, height);
    if (result === null) {
      res.status(404).send('no such asset');
    } else {
      // thumbnails will always be in JPEG format
      res.set({ 'Content-Type': 'image/jpeg' });
      // send() handles Content-Length and ETag support
      res.send(result);
    }
  } catch (err: any) {
    logger.error(err);
    res.redirect('/placeholder.svg');
  }
});

router.get('/raw/:id', async function (req, res, next) {
  const assetId = req.params.id;
  const filepath = blobs.blobPath(assetId);
  const asset = await records.getAssetById(assetId);
  const mimetype = asset.mediaType ? asset.mediaType : 'application/octet-stream';
  // sendFile() handles Content-Length, ETag, and range requests
  res.sendFile(filepath, { headers: { 'Content-Type': mimetype } }, function (err) {
    // only call next() if there is an error value
    if (err) {
      next(err);
    }
  });
});

export default router;
