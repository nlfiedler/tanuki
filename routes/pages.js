//
// Copyright (c) 2018 Nathan Fiedler
//
const express = require('express')
const path = require('path')
const favicon = require('serve-favicon')
const cookieParser = require('cookie-parser')
const bodyParser = require('body-parser')
const multer = require('multer')
const config = require('config')
const assets = require('lib/assets')
const backend = require('lib/backend')
const incoming = require('lib/incoming')

const router = express.Router()
const uploadPath = config.get('backend.uploadPath')
const upload = multer({ dest: uploadPath })

// wrapper that directs errors to the appropriate handler
let wrap = fn => (...args) => fn(...args).catch(args[2])

router.use(favicon(path.join(__dirname, '..', 'public', 'favicon.ico')))
router.use(bodyParser.json())
router.use(bodyParser.urlencoded({ extended: false }))
router.use(cookieParser())
const staticRoot = path.join(__dirname, '..', 'public')
router.use(express.static(staticRoot))

router.get('/thumbnail/:id', wrap(async function (req, res, next) {
  let id = req.params['id']
  let doc = await backend.fetchDocument(id)
  let mimetype = doc.mimetype ? doc.mimetype : 'application/octet-stream'
  let result = await assets.retrieveThumbnail(mimetype, id)
  if (result === null) {
    res.status(404).send('no such asset')
  } else {
    res.set({
      'Content-Type': result.mimetype,
      'ETag': id + '.thumb'
    })
    // res.send() handles Content-Length and cache freshness support
    res.send(result.binary)
  }
}))

router.get('/widethumb/:id', wrap(async function (req, res, next) {
  let id = req.params['id']
  let doc = await backend.fetchDocument(id)
  let mimetype = doc.mimetype ? doc.mimetype : 'application/octet-stream'
  let result = await assets.generateWideThumb(mimetype, id)
  if (result === null) {
    res.status(404).send('no such asset')
  } else {
    res.set({
      'Content-Type': result.mimetype,
      'ETag': id + '.wide'
    })
    // res.send() handles Content-Length and cache freshness support
    res.send(result.binary)
  }
}))

router.get('/preview/:id', wrap(async function (req, res, next) {
  let id = req.params['id']
  let doc = await backend.fetchDocument(id)
  let mimetype = doc.mimetype ? doc.mimetype : 'application/octet-stream'
  let result = await assets.generatePreview(mimetype, id)
  if (result === null) {
    res.status(404).send('no such asset')
  } else {
    res.set({
      'Content-Type': result.mimetype,
      'ETag': id + '.preview'
    })
    // res.send() handles Content-Length and cache freshness support
    res.send(result.binary)
  }
}))

router.get('/asset/:id', wrap(async function (req, res, next) {
  let id = req.params['id']
  let filepath = assets.assetPath(id)
  let doc = await backend.fetchDocument(id)
  let mimetype = doc.mimetype ? doc.mimetype : 'application/octet-stream'
  // res.sendFile() handles Content-Length and cache freshness support
  res.sendFile(filepath, {
    headers: {
      'Content-Type': mimetype,
      'ETag': id + '.asset'
    },
    immutable: true,
    maxAge: 86400000
  }, function (err) {
    // cannot unconditionally invoke the next handler...
    if (err) {
      next(err)
    }
  })
}))

router.post('/import', upload.single('asset'), wrap(async function (req, res, next) {
  let checksum = await incoming.computeChecksum(req.file.path)
  try {
    // check if an asset with this checksum already exists
    let assetId = await backend.byChecksum(checksum)
    if (assetId === null) {
      const originalDate = await incoming.getOriginalDate(req.file.mimetype, req.file.path)
      const importDate = Date.now()
      assetId = assets.makeAssetId(importDate, req.file.originalname)
      const duration = await assets.getDuration(req.file.mimetype, req.file.path)
      let doc = {
        _id: assetId,
        duration,
        filename: req.file.originalname,
        filesize: req.file.size,
        import_date: importDate,
        mimetype: req.file.mimetype,
        original_date: originalDate,
        checksum,
        // everything generally assumes the tags field is not undefined
        tags: []
      }
      await backend.updateDocument(doc)
    }
    // Ensure the asset is moved into position, just in case we managed
    // to commit the database record but failed to store the asset.
    await incoming.storeAsset(req.file.mimetype, req.file.path, assetId)
    res.redirect(`/assets/${assetId}/edit`)
  } catch (err) {
    // some other error occurred
    res.status(err.status).send(err.message)
  }
}))

// Last of all, map everything else to the web front-end.
router.get('/*', function (req, res, next) {
  res.render('index', { title: 'Browse Assets' })
})

module.exports = router
