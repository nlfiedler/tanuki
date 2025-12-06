//
// Copyright (c) 2025 Nathan Fiedler
//
import fs from 'node:fs/promises';
import express from 'express';
import multer from 'multer';
import container from 'tanuki/server/container.ts';
import logger from 'tanuki/server/logger.ts';

const settings = container.resolve('settingsRepository');
const dumpAssets = container.resolve('dumpAssets');
const loadAssets = container.resolve('loadAssets');

const uploads = settings.get('UPLOAD_PATH');
const storage = multer.diskStorage({
  destination: function (req, file, cb) {
    cb(null, uploads);
  }
});
const upload = multer({ storage: storage });

const router = express.Router();

// curl -F dump=@path/to/dump.json http://localhost:3000/records/load
router.post('/load', upload.single('dump'), async function (req, res, _next) {
  if (req.is('multipart/form-data')) {
    const file = req.file!;
    const raw = await fs.readFile(file.path, { encoding: 'utf-8' });
    const lines = raw.split(/\r?\n/).filter((ln) => ln.length > 0);
    const inputs = lines.map((ln) => JSON.parse(ln));
    await loadAssets(inputs);
    await fs.unlink(file.path);
    logger.info('loaded %d records', inputs.length);
    res.send({ ok: true, count: inputs.length });
  } else {
    logger.error(`content-type not valid: ${req.get('Content-Type')}`);
    res.status(400).send('Content-Type must be multipart/form-data');
  }
});

router.get('/dump', async function (req, res, next) {
  res.setHeader('Content-Type', 'application/json');
  //
  // Bun sets the Transfer-Encoding header when res.write() is called, and if
  // that header is already set it will result in a duplicate. Somehow that
  // confuses curl to the point that it fails to read anything and reports an
  // error (which itself seems odd).
  //
  // c.f. https://github.com/oven-sh/bun/issues/21201
  //
  // res.setHeader('Transfer-Encoding', 'chunked');
  let recordCount = 0;
  for await (const entry of dumpAssets(1024)) {
    const line = JSON.stringify(entry) + '\n';
    res.write(line);
    recordCount++;
  }
  res.end();
  logger.info('dumped %d records', recordCount);
});

export default router;
