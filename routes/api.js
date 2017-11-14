//
// Copyright (c) 2017 Nathan Fiedler
//
const express = require('express')
const backend = require('backend')
const router = express.Router()

// wrapper that directs errors to the appropriate handler
let wrap = fn => (...args) => fn(...args).catch(args[2])

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
