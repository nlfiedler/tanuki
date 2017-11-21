//
// Copyright (c) 2017 Nathan Fiedler
//
const express = require('express')
const backend = require('backend')
const router = express.Router()

// wrapper that directs errors to the appropriate handler
let wrap = fn => (...args) => fn(...args).catch(args[2])

// Return an integer given the input value. If value is nil, then return
// default. If value is an integer, return that, bounded by the minimum and
// maximum values. If value is a string, parse as an integer and ensure it
// falls within the minimum and maximum bounds.
function boundedIntValue (value, fallback, minimum, maximum) {
  let v = parseInt(value)
  return Math.min(Math.max(isNaN(v) ? fallback : v, minimum), maximum)
}

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
      let date = new Date(row['date']).toLocaleString()
      return {...row, date}
    })
    res.json({
      assets: formattedRows,
      // include total count of all matching rows
      count
    })
  }
}))

router.get('/assets/:id', wrap(async function (req, res, next) {
  try {
    let asset = await backend.fetchDocument(req.params['id'])
    res.json(asset)
  } catch (err) {
    // PouchDB errors are basically HTTP errors and work fine here.
    res.status(err.status).send(err.message)
  }
}))
// asset_path  POST  /api/assets      TanukiWeb.Web.AssetController :create
// asset_path  PUT   /api/assets/:id  TanukiWeb.Web.AssetController :update

router.get('/tags', wrap(async function (req, res, next) {
  let tags = await backend.allTags()
  res.json(tags)
}))

router.get('/locations', wrap(async function (req, res, next) {
  let locations = await backend.allLocations()
  res.json(locations)
}))

router.get('/years', wrap(async function (req, res, next) {
  let years = await backend.allYears()
  res.json(years)
}))

module.exports = router
