//
// Copyright (c) 2025 Nathan Fiedler
//
import express from 'express';
import multer from 'multer';
import container from 'tanuki/server/container.ts';
import logger from 'tanuki/server/logger.ts';

const settings = container.resolve('settingsRepository');
const importAsset = container.resolve('importAsset');

const uploads = settings.get('UPLOAD_PATH');
const storage = multer.diskStorage({
  destination: function (req, file, cb) {
    cb(null, uploads);
  }
});
const upload = multer({ storage: storage });

const router = express.Router();

router.post('/', upload.single('file_blob'), async function (req, res) {
  // req.file is the `file_blob` file object; req.body will hold any additional
  // fields provided in the request; the client is expected to provide the file
  // "last modified" time in Unix time as a string in 'last_modified'
  const modified = 'last_modified' in req.body ? new Date(parseInt(req.body.last_modified)) : new Date();
  const file = req.file!;
  const asset = await importAsset(file.path, file.originalname, file.mimetype, modified);
  logger.info(`asset ${file.originalname} imported as ${asset.key}`);
  res.redirect('/');
});

export default router;
