//
// Copyright (c) 2026 Nathan Fiedler
//
import express from 'express';
import container from 'tanuki/server/container.ts';
import logger from 'tanuki/server/logger.ts';

const faceStore: any = container.resolve('faceStore');

const router = express.Router();

// Serve the stored ~112px aligned face crop as a JPEG. The crop is produced at
// detection time and stored as a BLOB in the face store, so this is a single
// key lookup with no image processing.
router.get('/:id/thumb', async function (req, res, _next) {
  try {
    const bytes = await faceStore.faceThumbnail(req.params.id);
    if (!bytes) {
      res.redirect('/placeholder.svg');
      return;
    }
    res.set({ 'Content-Type': 'image/jpeg' });
    res.send(Buffer.from(bytes));
  } catch (error: any) {
    logger.error(error);
    res.redirect('/placeholder.svg');
  }
});

export default router;
