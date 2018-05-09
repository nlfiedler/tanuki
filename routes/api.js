//
// Copyright (c) 2018 Nathan Fiedler
//
const express = require('express')
const bodyParser = require('body-parser')
const multer = require('multer')
const config = require('config')
const backend = require('lib/backend')
const incoming = require('lib/incoming')
const assets = require('lib/assets')
const router = express.Router()

const uploadPath = config.get('backend.uploadPath')
const upload = multer({ dest: uploadPath })

router.use(bodyParser.json())

// wrapper that directs errors to the appropriate handler
let wrap = fn => (...args) => fn(...args).catch(args[2])

router.use(function (req, res, next) {
  // Disable automatic caching in express. See pending pull request:
  // https://github.com/expressjs/express/pull/2841
  req.headers['if-none-match'] = 'no-match-for-this'
  next()
})

router.post('/assets', upload.single('asset'), wrap(async function (req, res, next) {
  let id = await incoming.computeChecksum(req.file.path)
  try {
    // check if an asset with this identifier already exists
    await backend.fetchDocument(id)
    res.json({status: 'success', id: id})
  } catch (err) {
    if (err.status === 404) {
      const originalDate = await incoming.getOriginalDate(req.file.mimetype, req.file.path)
      const importDate = Date.now()
      const duration = await assets.getDuration(req.file.mimetype, req.file.path)
      let doc = {
        _id: id,
        duration,
        filename: req.file.originalname,
        filesize: req.file.size,
        import_date: importDate,
        mimetype: req.file.mimetype,
        original_date: originalDate,
        // everything generally assumes the tags field is not undefined
        tags: []
      }
      await backend.updateDocument(doc)
      await incoming.storeAsset(req.file.mimetype, req.file.path, id)
      res.json({status: 'success', id: id})
    } else {
      // some other error occurred
      res.status(err.status).send(err.message)
    }
  }
}))

module.exports = router
