//
// Copyright (c) 2017 Nathan Fiedler
//
const express = require('express')
const backend = require('backend')
const router = express.Router()

// wrapper that directs errors to the appropriate handler
let wrap = fn => (...args) => fn(...args).catch(args[2])

router.get('/assets', wrap(async function (req, res, next) {
  // TODO: param: locations=[a,b,c...]
  // TODO: param: years=[a,b,c...]
  // TODO: param: page=N
  // TODO: param: page_size=M (within range 1 to 100)
  // TODO: param: order (for sorting results)
  if (req.query['tags'] !== undefined) {
    let rows = await backend.byTags(req.query['tags'])
    // Perform some default sorting, with newer assets appearing earlier in the
    // list of results.
    rows.sort((a, b) => b['date'] - a['date'])
    let formattedRows = rows.map((row) => {
      let date = new Date(row['date']).toLocaleString()
      return {...row, date}
    })
    res.json({
      assets: formattedRows,
      // include total count of all matching rows
      count: rows.length
    })
  } else {
    // when no params are given, return count of assets
    let count = await backend.assetCount()
    res.json({
      assets: [],
      count
    })
  }
}))
// asset_path  GET   /api/assets/:id  TanukiWeb.Web.AssetController :show
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
