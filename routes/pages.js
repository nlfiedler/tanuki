//
// Copyright (c) 2017 Nathan Fiedler
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
  let checksum = req.params['id']
  let doc = await backend.fetchDocument(checksum)
  let mimetype = doc.mimetype ? doc.mimetype : 'application/octet-stream'
  let result = await assets.retrieveThumbnail(mimetype, checksum)
  if (result === null) {
    const imgFile = mimetypeToFilename(mimetype)
    res.sendFile(`images/${imgFile}`, {
      root: staticRoot
    }, function (err) {
      // cannot unconditionally invoke the next handler...
      if (err) {
        next(err)
      }
    })
  } else {
    res.set({
      'Content-Type': result.mimetype,
      'ETag': checksum + '.thumb'
    })
    // res.send() handles Content-Length and cache freshness support
    res.send(result.binary)
  }
}))

router.get('/preview/:id', wrap(async function (req, res, next) {
  let checksum = req.params['id']
  let doc = await backend.fetchDocument(checksum)
  let mimetype = doc.mimetype ? doc.mimetype : 'application/octet-stream'
  let result = await assets.generatePreview(mimetype, checksum)
  if (result === null) {
    const imgFile = mimetypeToFilename(mimetype)
    res.sendFile(`images/${imgFile}`, {
      root: staticRoot
    }, function (err) {
      // cannot unconditionally invoke the next handler...
      if (err) {
        next(err)
      }
    })
  } else {
    res.set({
      'Content-Type': result.mimetype,
      'ETag': checksum + '.preview'
    })
    // res.send() handles Content-Length and cache freshness support
    res.send(result.binary)
  }
}))

router.get('/asset/:id', wrap(async function (req, res, next) {
  let checksum = req.params['id']
  let filepath = assets.assetPath(checksum)
  let doc = await backend.fetchDocument(checksum)
  let mimetype = doc.mimetype ? doc.mimetype : 'application/octet-stream'
  // res.sendFile() handles Content-Length and cache freshness support
  res.sendFile(filepath, {
    headers: {
      'Content-Type': mimetype,
      'ETag': checksum + '.asset'
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
    await backend.fetchDocument(checksum)
    res.redirect(`/assets/${checksum}/edit`)
  } catch (err) {
    if (err.status === 404) {
      let originalDate = await incoming.getOriginalDate(req.file.mimetype, req.file.path)
      let importDate = incoming.dateToList(new Date())
      let doc = {
        _id: checksum,
        file_name: req.file.originalname,
        file_size: req.file.size,
        import_date: importDate,
        mimetype: req.file.mimetype,
        original_date: originalDate,
        // everything generally assumes the tags field is not undefined
        tags: []
      }
      await backend.updateDocument(doc)
      incoming.storeAsset(req.file.mimetype, req.file.path, checksum)
      res.redirect(`/assets/${checksum}/edit`)
    } else {
      // some other error occurred
      res.status(err.status).send(err.message)
    }
  }
}))

router.get('/upload', function (req, res, next) {
  res.render('upload', {title: 'Asset Upload'})
})

// Last of all, map everything else to the Elm appliation.
router.get('/*', function (req, res, next) {
  res.render('index', {title: 'Browse Assets'})
})

// Return the name of the an image to be used in place of the thumbnail
// for images that lack a thumbnail, for whatever reason.
function mimetypeToFilename (mimetype) {
  if (mimetype.startsWith('video/')) {
    return 'file-video-2.png'
  } else if (mimetype.startsWith('image/')) {
    return 'file-picture.png'
  } else if (mimetype.startsWith('audio/')) {
    return 'file-music-3.png'
  } else if (mimetype.startsWith('text/')) {
    return 'file-new-2.png'
  } else if (mimetype === 'application/pdf') {
    return 'file-acrobat.png'
  }
  return 'file-new-1.png'
}

module.exports = router
