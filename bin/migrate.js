//
// Copyright (c) 2018 Nathan Fiedler
//
const assets = require('../lib/assets')
const PouchDB = require('pouchdb')

const newDb = new PouchDB('/zeniba/shared/leveldb')
const oldDb = new PouchDB('http://localhost:5984/tanuki')

//
// Usage:
//
// $ node
// > const m = require('./bin/migrate')
// > m.migrate()
//
// old fields:     new fields:
//   caption         caption
//   exif_date       original_date
//   file_date       ---
//   file_name       file_name
//   file_owner      ---
//   file_size       file_size
//   import_date     import_date
//   location        location
//   mimetype        mimetype
//   sha256          _id
//   tags            tags
//   user_date       user_date
//

async function convertDocument (old) {
  const updated = Object.assign({}, old)
  // rename exif_date to original_date
  if (updated.hasOwnProperty('exif_date')) {
    updated.original_date = updated.exif_date
    delete updated.exif_date
  }
  // rename sha256 to _id
  updated._id = updated.sha256
  delete updated.sha256
  // remove file_date
  delete updated.file_date
  // remove file_owner
  delete updated.file_owner
  // get duration from videos
  const filepath = assets.assetPath(updated._id)
  const duration = await assets.getDuration(updated.mimetype, filepath)
  if (duration) {
    updated.duration = duration
  }
  return updated
}

async function updateDocumentAsync (newDoc) {
  try {
    // only insert new documents, ignoring existing ones
    await newDb.get(newDoc._id)
    return true
  } catch (err) {
    if (err.status === 404) {
      // the old _rev value is meaningless in the new database
      delete newDoc._rev
      await newDb.put(newDoc)
      return false
    } else {
      throw err
    }
  }
}

function migrateDocument (id) {
  oldDb.get(id).then(function (doc) {
    if (doc.sha256) {
      return convertDocument(doc)
    }
    throw new Error(`doc ${id} missing sha256 property`)
  }).then(function (newDoc) {
    return updateDocumentAsync(newDoc)
  }).then(function (res) {
    if (res) {
      console.info(`- ${id}`)
    } else {
      console.info(`+ ${id}`)
    }
  }).catch(function (err) {
    console.error(err)
  })
}

function migrate () {
  // With less than 4,000 documents at the time of migration, just fetch all of
  // the rows at one time.
  oldDb.allDocs().then(function (result) {
    //
    // basic shape of the result:
    //
    // { total_rows: 3740,
    //   offset: 0,
    //   rows:
    //    [ { id: '00ee2fe089100e34d59150e0b1000486',
    //        key: '00ee2fe089100e34d59150e0b1000486',
    //        value: { rev: '6-9f2e855077a893be73b99e160b5aa552' }
    //      },
    //      ...
    //    ]
    // }
    //
    for (let row of result.rows) {
      if (row.id.startsWith('_design')) {
        console.warn('ignoring design document:', row.id)
      } else {
        migrateDocument(row.id)
      }
    }
  })
}

module.exports = {
  migrate
}
