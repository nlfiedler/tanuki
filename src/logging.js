//
// Copyright (c) 2018 Nathan Fiedler
//

//
// TODO REMOVE WHEN issue #1130 FIXED
// https://github.com/winstonjs/winston/issues/1130
//
function clone (obj) {
  var copy = Array.isArray(obj) ? [] : {}
  for (var i in obj) {
    if (Array.isArray(obj[i])) {
      copy[i] = obj[i].slice(0)
    } else if (obj[i] instanceof Buffer) {
      copy[i] = obj[i].slice(0)
    } else if (typeof obj[i] !== 'function') {
      copy[i] = obj[i] instanceof Object ? clone(obj[i]) : obj[i]
    } else if (typeof obj[i] === 'function') {
      copy[i] = obj[i]
    }
  }
  return copy
}
require('winston/lib/winston/common').clone = clone

const Transport = require('winston-transport')
Transport.prototype.normalizeQuery = function (options) {
  options = options || {}

  // limit
  options.rows = options.rows || options.limit || 10

  // starting row offset
  options.start = options.start || 0

  // now
  options.until = options.until || new Date()
  if (typeof options.until !== 'object') {
    options.until = new Date(options.until)
  }

  // now - 24
  options.from = options.from || (options.until - (24 * 60 * 60 * 1000))
  if (typeof options.from !== 'object') {
    options.from = new Date(options.from)
  }

  // 'asc' or 'desc'
  options.order = options.order || 'desc'

  // which fields to select
  // options.fields = options.fields

  return options
}
Transport.prototype.formatResults = function (results, options) {
  return results
}
//
// END OF WORKAROUND
//

const winston = require('winston')
const config = require('config')

// Configure the logging not related to HTTP, which is handled elsewhere.
const transports = []
if (config.has('backend.logger.file')) {
  const filename = config.get('backend.logger.file')
  transports.push(new winston.transports.File({
    filename,
    maxsize: 1048576,
    maxFiles: 4
  }))
} else {
  transports.push(new winston.transports.Console())
}

let level = 'info'
if (config.has('backend.logger.level')) {
  level = config.get('backend.logger.level')
}

const logger = winston.createLogger({
  exitOnError: false,
  format: winston.format.json(),
  level,
  transports
})

module.exports = logger
