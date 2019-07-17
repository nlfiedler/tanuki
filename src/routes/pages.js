//
// Copyright (c) 2019 Nathan Fiedler
//
const express = require('express')
const path = require('path')
const favicon = require('serve-favicon')
const cookieParser = require('cookie-parser')
const bodyParser = require('body-parser')
const assets = require('assets')
const backend = require('backend')

const router = express.Router()

// wrapper that directs errors to the appropriate handler
const wrap = fn => (...args) => fn(...args).catch(args[2])

router.use(favicon(path.join(__dirname, '..', '..', 'public', 'favicon.ico')))
router.use(bodyParser.json())
router.use(bodyParser.urlencoded({ extended: false }))
router.use(cookieParser())
const staticRoot = path.join(__dirname, '..', '..', 'public')
router.use(express.static(staticRoot))

router.get('/thumbnail/:id', wrap(async function (req, res, next) {
  const id = req.params['id']
  const doc = await backend.fetchDocument(id)
  const mimetype = doc.mimetype ? doc.mimetype : 'application/octet-stream'
  const result = await assets.retrieveThumbnail(mimetype, id)
  if (result === null) {
    res.status(404).send('no such asset')
  } else {
    res.set({
      'Content-Type': result.mimetype,
      ETag: id + '.thumb'
    })
    // res.send() handles Content-Length and cache freshness support
    res.send(result.binary)
  }
}))

router.get('/widethumb/:id', wrap(async function (req, res, next) {
  const id = req.params['id']
  const doc = await backend.fetchDocument(id)
  const mimetype = doc.mimetype ? doc.mimetype : 'application/octet-stream'
  const result = await assets.generateWideThumb(mimetype, id)
  if (result === null) {
    res.status(404).send('no such asset')
  } else {
    res.set({
      'Content-Type': result.mimetype,
      ETag: id + '.wide'
    })
    // res.send() handles Content-Length and cache freshness support
    res.send(result.binary)
  }
}))

router.get('/preview/:id', wrap(async function (req, res, next) {
  const id = req.params['id']
  const doc = await backend.fetchDocument(id)
  const mimetype = doc.mimetype ? doc.mimetype : 'application/octet-stream'
  const result = await assets.generatePreview(mimetype, id)
  if (result === null) {
    res.status(404).send('no such asset')
  } else {
    res.set({
      'Content-Type': result.mimetype,
      ETag: id + '.preview'
    })
    // res.send() handles Content-Length and cache freshness support
    res.send(result.binary)
  }
}))

router.get('/asset/:id', wrap(async function (req, res, next) {
  const id = req.params['id']
  const filepath = assets.assetPath(id)
  const doc = await backend.fetchDocument(id)
  const mimetype = doc.mimetype ? doc.mimetype : 'application/octet-stream'
  // res.sendFile() handles Content-Length and cache freshness support
  res.sendFile(filepath, {
    headers: {
      'Content-Type': mimetype,
      ETag: id + '.asset'
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

// Last of all, map everything else to the web front-end.
router.get('/*', function (req, res, next) {
  res.render('index', { title: 'Browse Assets' })
})

module.exports = router
