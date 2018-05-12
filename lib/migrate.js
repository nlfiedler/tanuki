//
// Copyright (c) 2018 Nathan Fiedler
//
const _ = require('lodash')
const logger = require('winston')
const assets = require('lib/assets')

// Rename a property in the given object.
function renameProperty (obj, oldname, newname) {
  if (obj.hasOwnProperty(oldname)) {
    obj[newname] = obj[oldname]
    delete obj[oldname]
  }
}

// Remove the named property from the given object.
function removeProperty (obj, property) {
  if (obj.hasOwnProperty(property)) {
    delete obj[property]
  }
}

// Upgrade from version 0 to 1.
async function version0 (doc) {
  renameProperty(doc, 'exif_date', 'original_date')
  renameProperty(doc, 'sha256', '_id')
  removeProperty(doc, 'file_date')
  removeProperty(doc, 'file_owner')
  // get duration value from videos
  const filepath = assets.assetPath(doc._id)
  const duration = await assets.getDuration(doc.mimetype, filepath)
  if (duration) {
    doc.duration = duration
  }
}

// Upgrade from version 1 to 2.
function version1 (doc) {
  renameProperty(doc, 'file_name', 'filename')
  renameProperty(doc, 'file_size', 'filesize')
}

// Upgrade from version 2 to 3.
function version2 (doc) {
  // convert date/time int array [yyyy, mm, dd, HH, MM] to UTC milliseconds
  const fields = [
    'import_date',
    'original_date',
    'user_date'
  ]
  const tzMillisOffset = new Date().getTimezoneOffset() * 60000
  for (let property of fields) {
    if (doc.hasOwnProperty(property) && _.isArray(doc[property])) {
      const dl = doc[property]
      const utc = Date.UTC(dl[0], dl[1] - 1, dl[2], dl[3], dl[4]) + tzMillisOffset
      doc[property] = utc
    }
  }
}

// Upgrade from version 3 to 4.
function version3 (doc) {
  // no document changes in version 4
}

// Upgrade from version 4 to 5.
function version4 (doc) {
  // no document changes in version 5
}

const converters = [
  version0,
  version1,
  version2,
  version3,
  version4
]

// Migrate the database from some old version to the new one.
async function migrate (db, fromVersion, toVersion) {
  if (fromVersion > toVersion) {
    logger.warn('migrate from higher to lower version?')
    return false
  }
  logger.info('performing database migration...')
  // ideally this would fetch the results in chunks...
  let result = await db.allDocs()
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
    try {
      // Skip the design document itself, which the backend handles.
      if (!row.id.startsWith('_design')) {
        const oldDoc = await db.get(row.id)
        const newDoc = Object.assign({}, oldDoc)
        for (let version = fromVersion; version < toVersion; version++) {
          await converters[version](newDoc)
        }
        // write the new document, if it has actually changed
        if (!_.isEqual(newDoc, oldDoc)) {
          await db.put(newDoc)
        }
      }
    } catch (err) {
      logger.error('migrate error:', err)
      return false
    }
  }
  logger.info('database migration complete')
  return true
}

module.exports = {
  migrate
}
