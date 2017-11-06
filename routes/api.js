//
// Copyright (c) 2017 Nathan Fiedler
//
const express = require('express')
const backend = require('backend')
const router = express.Router()

router.get('/tags', function (req, res, next) {
  res.json(backend.allTags())
})

router.get('/locations', function (req, res, next) {
  res.json(backend.allLocations())
})

router.get('/years', function (req, res, next) {
  res.json(backend.allYears())
})

module.exports = router
