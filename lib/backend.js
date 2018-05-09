//
// Copyright (c) 2018 Nathan Fiedler
//
const config = require('config')
const fs = require('fs-extra')
const logger = require('winston')
const PouchDB = require('pouchdb')
const migrate = require('lib/migrate')

//
// Code for operating on the database.
//
// Basic information on the schema:
//
// * Asset record _id is the SHA256 checksum of the original asset.
//

/* global emit */

const dbPath = config.get('backend.dbPath')
fs.ensureDirSync(dbPath)
const db = new PouchDB(dbPath)

// Define the map/reduce query views.
let assetsDefinition = {
  _id: '_design/assets',
  // our monotonically increasing version number for tracking schema changes
  version: 3,
  views: {
    by_date: {
      map: function (doc) {
        let date = null
        if (doc.user_date) {
          date = doc.user_date
        } else if (doc.original_date) {
          date = doc.original_date
        } else {
          date = doc.import_date
        }
        let location = doc.location ? doc.location.toLowerCase() : ''
        // keep the included values the same across by_date, by_location, by_tag
        emit(date, [date, doc.filename, location])
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
          } else {
            date = doc.import_date
          }
          let location = doc.location.toLowerCase()
          // keep the included values the same across by_date, by_location, by_tag
          emit(location, [date, doc.filename, location])
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
          } else {
            date = doc.import_date
          }
          let location = doc.location ? doc.location.toLowerCase() : ''
          doc.tags.forEach(function (tag) {
            // keep the included values the same across by_date, by_location, by_tag
            emit(tag.toLowerCase(), [date, doc.filename, location])
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
          emit(new Date(doc.user_date).getFullYear(), 1)
        } else if (doc.original_date) {
          emit(new Date(doc.original_date).getFullYear(), 1)
        } else {
          emit(new Date(doc.import_date).getFullYear(), 1)
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
      let ok = await migrate.migrate(db, oldDoc.version || 0, index.version)
      if (ok) {
        await db.put({...index, _rev: oldDoc._rev})
        created = true
      }
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
    logger.info('database indices created')
    await primeIndices(assetsDefinition)
    logger.info('database indices primed')
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
  logger.info(`removed all ${results.length} documents`)
  // ensure the now stale views are removed as well
  await db.viewCleanup()
  return initDatabase()
}

/**
 * Update the existing document, if any, or insert as a new document.
 *
 * @param {object} newDoc - new document
 * @param {string} newDoc._id - document identifier
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
  // let any errors bubble up to the caller
  return updateDocumentAsync(newDoc).then(function (res) {
    let action = res ? 'updated existing' : 'inserted new'
    logger.info(`${action} document ${newDoc._id}`)
  })
}

/**
 * Retrieve the document with the given identifier.
 *
 * @param {string} docId - identifier of document to retrieve.
 * @returns {Promise<Object>} Promise resolving to document object.
 */
async function fetchDocument (docId) {
  return db.get(docId)
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
    logger.error('allTags error:', err)
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
    logger.error('allLocations error:', err)
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
    logger.error('allYears error:', err)
    return []
  })
}

/**
 * Return the number of assets stored in the database.
 *
 * @returns {Promise<number>} Promise resolving to total count of assets.
 */
async function assetCount () {
  let allDocs = await db.allDocs()
  // Count those documents that have id starting with "_design/" then subtract
  // that from the total_rows to find the true asset count.
  const designCount = allDocs.rows.reduce((acc, row) => {
    return row.id.startsWith('_design/') ? acc + 1 : acc
  }, 0)
  return allDocs.total_rows - designCount
}

// Convert the raw results from the database query into something resembling
// what the client would expect.
function buildSummaries (queryResults) {
  let results = queryResults.map((row) => {
    return {
      id: row['id'],
      datetime: row['value'][0],
      filename: row['value'][1],
      location: row['value'][2]
    }
  })
  return results
}

// Find all assets that do not have a location value.
async function withoutLocation () {
  let queryResults = await db.allDocs()
  let results = []
  for (let row of queryResults.rows) {
    try {
      // skip any design documents
      if (!row.id.startsWith('_design')) {
        const doc = await db.get(row.id)
        if (!doc.location) {
          results.push({
            id: row.id,
            datetime: getBestDate(doc),
            filename: doc.filename,
            location: doc.location
          })
        }
      }
    } catch (err) {
      console.error('withoutLocation error:', err)
    }
  }
  return results
}

/**
 * Retrieve asset summaries for those assets whose location matches one of
 * those given.
 *
 * @param {Array<string>} location - list of locations to query.
 * @returns {Promise<Array>} Promise resolving to list of asset summaries.
 */
async function byLocations (locations) {
  // special case to find assets with no location at all
  if (locations.length === 1 && locations[0] === null) {
    return withoutLocation()
  }
  let queryResults = await db.query('assets/by_location', {
    keys: Array.from(locations).sort()
  })
  return buildSummaries(queryResults.rows)
}

/**
 * Retrieve asset summaries for those assets whose year matches the one given.
 *
 * Either after or before may be null, but not both.
 *
 * @param {Number} after - date of oldest asset to include in results.
 * @param {Number} before - date of newest asset to include in results.
 * @param {boolean} summarize - true to convert results to summaries, false to leave in raw form.
 * @returns {Promise<Array>} Promise resolving to list of asset summaries.
 */
async function byDateRange (after, before, summarize = true) {
  const keys = after ? (before ? {
    start_key: after,
    end_key: before
  } : {
    start_key: after
  }) : {
    end_key: before
  }
  let queryResults = await db.query('assets/by_date', keys)
  return summarize ? buildSummaries(queryResults.rows) : queryResults.rows
}

// Find all assets that do not have any tags at all.
async function withoutTags () {
  let queryResults = await db.allDocs()
  let results = []
  for (let row of queryResults.rows) {
    try {
      // skip any design documents
      if (!row.id.startsWith('_design')) {
        const doc = await db.get(row.id)
        if (!doc.tags || doc.tags.length === 0) {
          results.push({
            id: row.id,
            datetime: getBestDate(doc),
            filename: doc.filename,
            location: doc.location
          })
        }
      }
    } catch (err) {
      console.error('withoutTags error:', err)
    }
  }
  return results
}

/**
 * Retrieve asset summaries for those assets whose tags match those given.
 *
 * @param {Array} tags - list of tags by which to query.
 * @param {boolean} summarize - true to convert results to summaries, false to leave in raw form.
 * @returns {Promise<Array>} Promise resolving to list of asset summaries.
 */
async function byTags (tags, summarize = true) {
  // special case to find assets with no tags at all
  if (tags.length === 1 && tags[0] === null) {
    return withoutTags()
  }
  let queryResults = await db.query('assets/by_tag', {
    keys: Array.from(tags).sort()
  })
  // Reduce the results to those that have all of the given tags.
  let tagCounts = queryResults.rows.reduce((acc, row) => {
    let docId = row['id']
    let count = acc.has(docId) ? acc.get(docId) : 0
    acc.set(docId, count + 1)
    return acc
  }, new Map())
  let matchingRows = queryResults.rows.filter((row) => {
    return tagCounts.get(row['id']) === tags.length
  })
  // Remove the duplicate rows by sorting on the document identifier and
  // removing any duplicates.
  let rawResults = matchingRows.sort((a, b) => {
    return a['id'].localeCompare(b['id'])
  }).filter((row, idx, arr) => idx === 0 || row['id'] !== arr[idx - 1]['id'])
  return summarize ? buildSummaries(rawResults) : rawResults
}

// Filter query results by the given locations.
function filterByLocation (locations, rows) {
  return rows.filter((row) => locations.findIndex((l) => l === row['value'][2]) !== -1)
}

/**
 * Filter query results by the given date range. If both after and before
 * are null, the rows are returned unfiltered.
 *
 * @param {Number} after - date of oldest asset to include in results.
 * @param {Number} before - date of newest asset to include in results.
 * @param {Array} rows - query results to be filtered by date.
 * @returns {Array} results that are within the given range.
 */
function filterByDateRange (after, before, rows) {
  if (after && before) {
    return rows.filter((row) => {
      return row['value'][0] > after && row['value'][0] < before
    })
  }
  if (after) {
    return rows.filter((row) => {
      return row['value'][0] > after
    })
  }
  if (before) {
    return rows.filter((row) => {
      return row['value'][0] < before
    })
  }
  return rows
}

/**
 * Query assets by tags, or years, or locations, or any combination of
 * those. Each argument is a list of values on which to select assets.
 *
 * @param {Array} tags - list of tags to search for, if any.
 * @param {Array} locations - list of locations to search for, if any.
 * @param {Number} after - include assets whose date is after this date.
 * @param {Number} before - include assets whose date is before this date.
 * @returns {Promise<Array>} Promise resolving to list of asset summaries.
 */
async function query (tags, locations, after, before) {
  // poor man's pattern matching...
  let b3 = tags && tags.length ? 4 : 0
  let b2 = after || before ? 2 : 0
  let b1 = locations && locations.length ? 1 : 0
  switch (b3 | b2 | b1) {
    case 0:
      return []
    case 4:
      return byTags(tags)
    case 2:
      return byDateRange(after, before)
    case 1:
      return byLocations(locations)
    case 6: {
      let unfiltered = await byTags(tags, false)
      return buildSummaries(filterByDateRange(after, before, unfiltered))
    }
    case 3: {
      let unfiltered = await byDateRange(after, before, false)
      return buildSummaries(filterByLocation(locations, unfiltered))
    }
    case 5: {
      let unfiltered = await byTags(tags, false)
      return buildSummaries(filterByLocation(locations, unfiltered))
    }
    case 7: {
      let unfiltered = await byTags(tags, false)
      let filtered = filterByDateRange(after, before, unfiltered)
      return buildSummaries(filterByLocation(locations, filtered))
    }
  }
}

// The field names of the date/time values in their preferred order. That is,
// the user-provided value is considered the best, with the Exif original being
// second, and so on.
const bestDateOrder = [
  'user_date',
  'original_date',
  'import_date'
]

// Retrieve the preferred date/time value from the document.
function getBestDate (doc) {
  for (let field of bestDateOrder) {
    if (field in doc && doc[field]) {
      return doc[field]
    }
  }
  return null
}

module.exports = {
  allLocations,
  allTags,
  allYears,
  assetCount,
  byLocations,
  byTags,
  byDateRange,
  fetchDocument,
  initDatabase,
  reinitDatabase,
  query,
  updateDocument,
  getBestDate
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
