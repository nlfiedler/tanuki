//
// Copyright (c) 2017 Nathan Fiedler
//
const config = require('config')
const fs = require('fs-extra')
const PouchDB = require('pouchdb')

/* global emit */

const dbPath = config.get('backend.dbPath')
fs.ensureDirSync(dbPath)
const db = new PouchDB(dbPath)

// Define the map/reduce query views.
let assetsDefinition = {
  _id: '_design/assets',
  // our monotonically increasing version number for tracking schema changes
  version: 1,
  views: {
    by_date: {
      map: function (doc) {
        let date = null
        if (doc.user_date) {
          date = doc.user_date
        } else if (doc.original_date) {
          date = doc.original_date
        } else if (doc.file_date) {
          date = doc.file_date
        } else {
          date = doc.import_date
        }
        let location = doc.location ? doc.location.toLowerCase() : ''
        // keep the included values the same across by_date, by_location, by_tag
        emit(date, [date, doc.file_name, doc._id, location])
      }.toString()
    },
    by_location: {
      map: function (doc) {
        if (doc.location) {
          let date = null
          if (doc.user_date) {
            date = doc.user_date
          } else if (doc.original_date) {
            date = doc.original_date
          } else if (doc.file_date) {
            date = doc.file_date
          } else {
            date = doc.import_date
          }
          let location = doc.location.toLowerCase()
          // keep the included values the same across by_date, by_location, by_tag
          emit(location, [date, doc.file_name, doc._id, location])
        }
      }.toString()
    },
    by_tag: {
      map: function (doc) {
        if (doc.tags && Array.isArray(doc.tags)) {
          let date = null
          if (doc.user_date) {
            date = doc.user_date
          } else if (doc.original_date) {
            date = doc.original_date
          } else if (doc.file_date) {
            date = doc.file_date
          } else {
            date = doc.import_date
          }
          let location = doc.location ? doc.location.toLowerCase() : ''
          doc.tags.forEach(function (tag) {
            // keep the included values the same across by_date, by_location, by_tag
            emit(tag.toLowerCase(), [date, doc.file_name, doc._id, location])
          })
        }
      }.toString()
    },
    locations: {
      map: function (doc) {
        if (doc.location) {
          emit(doc.location.toLowerCase(), 1)
        }
      }.toString(),
      reduce: '_count'
    },
    tags: {
      map: function (doc) {
        if (doc.tags && Array.isArray(doc.tags)) {
          doc.tags.forEach(function (tag) {
            emit(tag.toLowerCase(), 1)
          })
        }
      }.toString(),
      reduce: '_count'
    },
    years: {
      map: function (doc) {
        if (doc.user_date) {
          emit(doc.user_date[0], 1)
        } else if (doc.original_date) {
          emit(doc.original_date[0], 1)
        } else if (doc.file_date) {
          emit(doc.file_date[0], 1)
        } else {
          emit(doc.import_date[0], 1)
        }
      }.toString(),
      reduce: '_count'
    }
  }
}

/**
 * If the schema has changed, update the design document.
 * If it was not yet created, do so now.
 *
 * @param {object} index - design document to be inserted/updated.
 * @param {string} index._id - document identifier
 * @param {number} index.version - design view version number.
 * @param {object} index.views - Map of view definitions.
 * @returns {Promise<boolean>} true if index was created, false otherwise.
 */
async function createIndices (index) {
  let created = false
  try {
    let oldDoc = await db.get(index._id)
    if (oldDoc.version === undefined || oldDoc.version < index.version) {
      await db.put({...index, _rev: oldDoc._rev})
      created = true
    }
  } catch (err) {
    if (err.status === 404) {
      await db.put(index)
      created = true
    } else {
      throw err
    }
  }
  return created
}

/**
 * Perform a query against all of the views to prime the indices.
 *
 * @param {object} index - design document to be primed.
 * @param {object} index.views - Map of view definitions.
 * @returns {Promise<Array>} of query results, without any row data.
 */
async function primeIndices (index) {
  let promises = []
  for (const view in index.views) {
    promises.push(db.query(`assets/${view}`, {
      limit: 0
    }))
  }
  return Promise.all(promises)
}

/**
 * Ensure the database is prepared with the necessary design documents.
 *
 * @returns {Promise<string>} 'ok'
 */
async function initDatabase () {
  let indexCreated = await createIndices(assetsDefinition)
  if (indexCreated) {
    console.info('database indices created')
  }
  if (indexCreated) {
    await primeIndices(assetsDefinition)
    console.info('database indices primed')
  }
  return 'ok'
}

/**
 * Delete all existing documents, including the design documents, and
 * initialize everything again. Primarily used in testing.
 *
 * @returns {Promise<string>} 'ok'
 */
async function reinitDatabase () {
  let allDocs = await db.allDocs({include_docs: true})
  let promises = allDocs.rows.map((row) => db.remove(row.doc))
  let results = await Promise.all(promises)
  console.info(`removed all ${results.length} documents`)
  // ensure the now stale views are removed as well
  await db.viewCleanup()
  return initDatabase()
}

/**
 * Update the existing document, if any, or insert as a new document.
 *
 * @returns {Promise<Boolean>} true if document was updated, false if inserted.
 */
async function updateDocumentAsync (newDoc) {
  try {
    let oldDoc = await db.get(newDoc._id)
    await db.put({...newDoc, _rev: oldDoc._rev})
    return true
  } catch (err) {
    if (err.status === 404) {
      await db.put(newDoc)
      return false
    } else {
      throw err
    }
  }
}

/**
 * Insert or update a document in the database, asynchronously.
 *
 * @param {object} newDoc - new document
 * @param {string} newDoc._id - document identifier
 * @returns {Promise} resolving to undefined
 */
function updateDocument (newDoc) {
  return updateDocumentAsync(newDoc).then(function (res) {
    let action = res ? 'updated existing' : 'inserted new'
    console.info(`${action} document ${newDoc._id}`)
  }).catch(function (err) {
    console.error(err)
  })
}

/**
 * Retrieves all of the tags, as an array of objects.
 *
 * @returns {Promise<Array>} Promise resolving to array of tag objects.
 */
function allTags () {
  return db.query('assets/tags', {
    group_level: 1
  }).then(function (res) {
    return res['rows']
  }).catch(function (err) {
    console.error('allTags error:', err)
    return []
  })
}

/**
 * Retrieves all of the locations, as an array of objects.
 *
 * @returns {Promise<Array>} Promise resolving to array of location objects.
 */
function allLocations () {
  return db.query('assets/locations', {
    group_level: 1
  }).then(function (res) {
    return res['rows']
  }).catch(function (err) {
    console.error('allLocations error:', err)
    return []
  })
}

/**
 * Retrieves all of the years, as an array of objects.
 *
 * @returns {Promise<Array>} Promise resolving to array of year objects.
 */
function allYears () {
  return db.query('assets/years', {
    group_level: 1
  }).then(function (res) {
    return res['rows']
  }).catch(function (err) {
    console.error('allYears error:', err)
    return []
  })
}

module.exports = {
  allLocations,
  allTags,
  allYears,
  initDatabase,
  reinitDatabase,
  updateDocument
}

//
// code for implementing config record, with defaults
//
// db.get('config').catch(function (err) {
//   if (err.name === 'not_found') {
//     return {
//       _id: 'config',
//       background: 'blue',
//       foreground: 'white',
//       sparkly: 'false'
//     };
//   } else { // hm, some other error
//     throw err;
//   }
// }).then(function (configDoc) {
//   // sweet, here is our configDoc
// }).catch(function (err) {
//   // handle any errors
// });
