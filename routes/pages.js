//
// Copyright (c) 2017 Nathan Fiedler
//
const express = require('express')
const path = require('path')
const favicon = require('serve-favicon')
const cookieParser = require('cookie-parser')
const bodyParser = require('body-parser')
const router = express.Router()

router.use(favicon(path.join(__dirname, '..', 'public', 'favicon.ico')))
router.use(bodyParser.json())
router.use(bodyParser.urlencoded({ extended: false }))
router.use(cookieParser())
router.use(express.static(path.join(__dirname, '..', 'public')))

// GET home page
router.get('/', function (req, res, next) {
  res.render('index', {title: 'Tanuki'})
})

module.exports = router
