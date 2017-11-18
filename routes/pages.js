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

// page_path  GET   /upload         TanukiWeb.Web.PageController :upload
// page_path  POST  /import         TanukiWeb.Web.PageController :import
// page_path  GET   /asset/:id      TanukiWeb.Web.PageController :asset
// page_path  GET   /thumbnail/:id  TanukiWeb.Web.PageController :thumbnail
// page_path  GET   /preview/:id    TanukiWeb.Web.PageController :preview
// page_path  GET   /*path          TanukiWeb.Web.PageController :index

// GET home page
router.get('/', function (req, res, next) {
  res.render('index', {title: 'Tanuki'})
})

module.exports = router
