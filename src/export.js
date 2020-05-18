//
// Copyright (c) 2020 Nathan Fiedler
//
const config = require('config')
const fs = require('fs-extra')
const logger = require('./logging')
const PouchDB = require('pouchdb')

const dbPath = config.get('backend.dbPath')
fs.ensureDirSync(dbPath)
const db = new PouchDB(dbPath)
logger.info('beginning export...')
db.allDocs({
  include_docs: true
}).then(function (result) {
  logger.info('read all documents')
  // filter out the design view things, not memory efficient
  const rows = result.rows.filter((row) => {
    return !row.id.startsWith('_design')
  })
  return new Promise((resolve, reject) => {
    const json = JSON.stringify(rows, null, 2)
    fs.writeFile('dump.json', json, 'utf-8', (err) => {
      if (err) {
        reject(err)
      } else {
        resolve()
      }
    })
  })
}).then(() => {
  logger.info('finished export')
}).catch(function (err) {
  logger.error('error in allDocs()', err)
})
