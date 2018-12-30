//
// Copyright (c) 2018 Nathan Fiedler
//
const _ = require('lodash')
const config = require('config')
const fs = require('fs-extra')
const logger = require('lib/logging')
const path = require('path')
const assets = require('lib/assets')

const assetsPath = config.get('backend.assetPath')

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

// Upgrade document to version 1.
async function dataVersion1 (doc) {
  if (!doc.hasOwnProperty('file_date')) {
    // avoid renaming the sha256 field as later versions have that field again
    return
  }
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

// Upgrade document to version 2.
function dataVersion2 (doc) {
  renameProperty(doc, 'file_name', 'filename')
  renameProperty(doc, 'file_size', 'filesize')
}

// Upgrade document to version 3.
function dataVersion3 (doc) {
  // convert date/time int array [yyyy, mm, dd, HH, MM] to UTC milliseconds
  const fields = [
    'import_date',
    'original_date',
    'user_date'
  ]
  const tzMillisOffset = new Date().getTimezoneOffset() * 60000
  for (let property of fields) {
    if (doc.hasOwnProperty(property) && Array.isArray(doc[property])) {
      const dl = doc[property]
      const utc = Date.UTC(dl[0], dl[1] - 1, dl[2], dl[3], dl[4]) + tzMillisOffset
      doc[property] = utc
    }
  }
}

// Upgrade document to version 7.
function dataVersion7 (doc) {
  //
  // Skip any document that already has the sha256 field (again), as this
  // particular function is _not_ idempotent.
  //
  // And does not have the 'checksum' field, too!
  //
  // Since the _id is changing the _rev field must be cleared so the document
  // will be saved anew.
  //
  if (!doc.hasOwnProperty('sha256') && !doc.hasOwnProperty('checksum')) {
    renameProperty(doc, '_id', 'sha256')
    doc._id = assets.makeAssetId(doc.import_date, doc.filename)
    removeProperty(doc, '_rev')
  }
}

// Upgrade document to version 8.
function dataVersion8 (doc) {
  // rename and field and add a prefix
  if (doc.hasOwnProperty('sha256')) {
    doc['checksum'] = 'sha256-' + doc['sha256']
    delete doc['sha256']
  }
}

// Perform post-commit actions for version 7.
function postVersion7 (doc) {
  //
  // Perform the asset path migration. This involves moving the assets to a
  // new directory structure that is based on the import date/time. The old
  // asset path was based on the checksum.
  //
  // Since version 8, the checksum has an algorithm prefix (e.g. 'sha224-*')
  //
  const digest = doc.checksum.split('-')[1]
  const part1 = digest.slice(0, 2)
  const part2 = digest.slice(2, 4)
  const part3 = digest.slice(4)
  let asp = path.join(assetsPath, part1, part2, part3)
  const srcpath = path.isAbsolute(asp) ? asp : path.join(process.cwd(), asp)
  const destpath = assets.assetPath(doc._id)
  // if the new path doesn't exist and the old one does, rename it
  if (!fs.existsSync(destpath) && fs.existsSync(srcpath)) {
    fs.ensureDirSync(path.dirname(destpath))
    fs.renameSync(srcpath, destpath)
  }
  // video thumbnails need to move, too
  const srcThumb = srcpath + '.jpg'
  if (fs.existsSync(srcThumb)) {
    const dirname = path.dirname(destpath)
    const basename = path.basename(destpath, path.extname(destpath))
    const destThumb = path.join(dirname, basename + '.jpg')
    fs.renameSync(srcThumb, destThumb)
  }
  try {
    // remove old parent directories as they become empty
    fs.rmdirSync(path.dirname(srcpath))
    fs.rmdirSync(path.dirname(path.dirname(srcpath)))
  } catch (err) {
    // these errors are expected, log everything else
    const code = 'code' in err ? err.code : null
    if (code !== 'ENOTEMPTY' && code !== 'ENOENT') {
      logger.error(err)
    }
  }
}

// List of database converters. Not all versions have document changes.
const dataConverters = [
  dataVersion1,
  dataVersion2,
  dataVersion3,
  null, // 4
  null, // 5
  null, // 6
  dataVersion7,
  dataVersion8
]

// List of post-commit migrators. Most versions do not involve changes
// external to the database.
//
// N.B. post conversions run after _all_ data migrations have occurred,
// so the document schema is the latest version at the time these run.
const postCommits = [
  null, // 1
  null, // 2
  null, // 3
  null, // 4
  null, // 5
  null, // 6
  postVersion7,
  null // 8
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
        // run the document converters
        for (let version = fromVersion; version < toVersion; version++) {
          if (dataConverters[version] !== null) {
            await dataConverters[version](newDoc)
          }
        }
        // write the new document, if it has actually changed
        if (!_.isEqual(newDoc, oldDoc)) {
          await db.put(newDoc)
          if (!newDoc.hasOwnProperty('_rev') && oldDoc._id !== newDoc._id) {
            logger.info(`document ${newDoc._id} replaces ${oldDoc._id}`)
            // If the new document does not have a revision, and the identifier
            // is different, that means the migration basically changed the
            // entire document. Remove the old one, it is irrelevant now.
            await db.remove(oldDoc)
          }
        }
        // run post-commit migrators, which handle non-document operations
        for (let version = fromVersion; version < toVersion; version++) {
          if (postCommits[version] !== null) {
            await postCommits[version](newDoc)
          }
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
