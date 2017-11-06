//
// Copyright (c) 2017 Nathan Fiedler
//
require('app-module-path').addPath(__dirname)
const express = require('express')
const path = require('path')
const logger = require('morgan')
const apiRoutes = require('routes/api')
const pageRoutes = require('routes/pages')

const app = express()

// view engine setup
app.set('views', path.join(__dirname, 'views'))
app.set('view engine', 'ejs')

app.use(logger('dev'))

app.use('/api', apiRoutes)
app.use('/', pageRoutes)

// catch 404 and forward to error handler
app.use((req, res, next) => {
  let err = new Error('Not Found')
  err.status = 404
  next(err)
})

// error handler
app.use((err, req, res, next) => {
  // set locals, only providing error in development
  res.locals.message = err.message
  res.locals.error = req.app.get('env') === 'development' ? err : {}

  // render the error page
  res.status(err.status || 500)
  res.render('error')
})

module.exports = app
