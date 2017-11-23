//
// Copyright (c) 2017 Nathan Fiedler
//
require('app-module-path').addPath(__dirname)
const express = require('express')
const path = require('path')
const logger = require('morgan')
const apiRoutes = require('routes/api')
const pageRoutes = require('routes/pages')
const backend = require('lib/backend')
const config = require('config')
const winston = require('winston')
const rfs = require('rotating-file-stream')

// Configure the logging not related to HTTP, which is handled using morgan.
winston.exitOnError = false
winston.level = config.get('backend.logger.level')
if (config.has('backend.logger.file')) {
  const filename = config.get('backend.logger.file')
  winston.add(winston.transports.File, {
    filename,
    maxsize: 1048576,
    maxFiles: 4
  })
  winston.remove(winston.transports.Console)
}

// Initialize the database asynchronously.
backend.initDatabase().then(function (res) {
  winston.info('database initialization result:', res)
}).catch(function (err) {
  winston.error('database initialization error:', err)
})

const app = express()

// view engine setup
app.set('views', path.join(__dirname, 'views'))
app.set('view engine', 'ejs')

// Configure the HTTP logging.
if (config.has('morgan.logger.logPath')) {
  const logDirectory = config.get('morgan.logger.logPath')
  const accessLogStream = rfs('access.log', {
    size: '1M',
    maxFiles: 4,
    path: logDirectory
  })
  app.use(logger('combined', {stream: accessLogStream}))
} else {
  app.use(logger('dev'))
}

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
