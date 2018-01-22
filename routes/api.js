//
// Copyright (c) 2017 Nathan Fiedler
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

// Ensure the years parameter can be parsed as integers.
router.use(function (req, res, next) {
  if (req.query['years'] !== undefined) {
    let years = req.query['years']
    if (!Array.isArray(years)) {
      res.status(400).send('years must be an array')
    } else {
      let intYears = years.map(y => parseInt(y))
      if (intYears.some(y => isNaN(y))) {
        res.status(400).send('years must be integers')
      } else {
        req.queryYears = intYears
        next()
      }
    }
  } else {
    next()
  }
})

router.get('/assets', wrap(async function (req, res, next) {
  // TODO: param: order (for sorting results)
  let tags = req.query['tags']
  let years = req.queryYears
  let locations = req.query['locations']
  if (!tags && !years && !locations) {
    // when no params are given, return count of assets
    const count = await backend.assetCount()
    res.json({
      assets: [],
      count
    })
  } else if (tags && !Array.isArray(tags)) {
    res.status(400).send('tags must be an array')
  } else if (locations && !Array.isArray(locations)) {
    res.status(400).send('locations must be an array')
  } else {
    let rows = await backend.query(tags, years, locations)
    // Perform some default sorting, with newer assets appearing earlier in the
    // list of results.
    rows.sort((a, b) => b['date'] - a['date'])
    // count is the number of _all_ matching results
    const count = rows.length
    // handle pagination with certain defaults and bounds
    const pageSize = boundedIntValue(req.query['page_size'], 10, 1, 100)
    const pageLimit = Math.ceil(count / pageSize)
    const page = boundedIntValue(req.query['page'], 1, 1, pageLimit)
    const start = (page - 1) * pageSize
    let pageRows = rows.slice(start, start + pageSize)
    let formattedRows = pageRows.map((row) => {
      // the summary includes only the formatted date, no time
      let date = new Date(row['date']).toLocaleDateString()
      return {...row, date}
    })
    res.json({
      assets: formattedRows,
      // include total count of all matching rows
      count
    })
  }
}))

router.post('/assets', upload.single('asset'), wrap(async function (req, res, next) {
  let checksum = await incoming.computeChecksum(req.file.path)
  try {
    // check if an asset with this checksum already exists
    await backend.fetchDocument(checksum)
    res.json({status: 'success', id: checksum})
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
      res.json({status: 'success', id: checksum})
    } else {
      // some other error occurred
      res.status(err.status).send(err.message)
    }
  }
}))

router.get('/assets/:id', wrap(async function (req, res, next) {
  try {
    let asset = await backend.fetchDocument(req.params['id'])
    let defaults = {
      caption: null,
      location: null
    }
    res.json({
      ...defaults,
      ...asset,
      checksum: asset['_id'],
      datetime: dateListToString(assets.getBestDate(asset)),
      duration: await assets.getDuration(asset.mimetype, asset['_id']),
      user_date: dateListToString(asset['user_date'])
    })
  } catch (err) {
    // PouchDB errors are basically HTTP errors and work fine here.
    res.status(err.status).send(err.message)
  }
}))

router.put('/assets/:id', wrap(async function (req, res, next) {
  try {
    const asset = await backend.fetchDocument(req.params['id'])
    // merge the new values into the existing document
    const updated = incoming.updateAssetFields(asset, req.body)
    await backend.updateDocument(updated)
    res.json({status: 'success'})
  } catch (err) {
    // PouchDB errors are basically HTTP errors and work fine here.
    res.status(err.status).send(err.message)
  }
}))

router.get('/tags', wrap(async function (req, res, next) {
  let tags = await backend.allTags()
  // convert the field names to something sensible
  let renamed = tags.map((v) => {
    return {tag: v.key, count: v.value}
  })
  res.json(renamed)
}))

router.get('/locations', wrap(async function (req, res, next) {
  let locations = await backend.allLocations()
  // convert the field names to something sensible
  let renamed = locations.map((v) => {
    return {location: v.key, count: v.value}
  })
  res.json(renamed)
}))

router.get('/years', wrap(async function (req, res, next) {
  let years = await backend.allYears()
  // convert the field names to something sensible
  let renamed = years.map((v) => {
    return {year: v.key, count: v.value}
  })
  res.json(renamed)
}))

// Return an integer given the input value. If value is nil, then return
// default. If value is an integer, return that, bounded by the minimum and
// maximum values. If value is a string, parse as an integer and ensure it
// falls within the minimum and maximum bounds.
function boundedIntValue (value, fallback, minimum, maximum) {
  let v = parseInt(value)
  return Math.min(Math.max(isNaN(v) ? fallback : v, minimum), maximum)
}

// Convert the array of integers to a formatted date/time string.
function dateListToString (dl) {
  if (!dl) {
    return ''
  }
  // need to adjust the month for Date object
  return new Date(dl[0], dl[1] - 1, dl[2], dl[3], dl[4]).toLocaleString()
}

module.exports = router
