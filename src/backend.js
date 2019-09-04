//
// Copyright (c) 2019 Nathan Fiedler
//
const config = require('config')
const fs = require('fs-extra')
const logger = require('logging')
const PouchDB = require('pouchdb')
PouchDB.plugin(require('pouchdb-find'))
const migrate = require('migrate')

//
// Code for operating on the database.
//

/* global emit */

const dbPath = config.get('backend.dbPath')
fs.ensureDirSync(dbPath)
let db = new PouchDB(dbPath)

// Define the map/reduce query views.
const assetsDefinition = {
  _id: '_design/assets',
  // our monotonically increasing version number for tracking schema changes
  version: 8,
  views: {
    by_checksum: {
      map: function (doc) {
        // special index, only need the identifier for the checksum
        if (doc.checksum) {
          emit(doc.checksum.toLowerCase())
        }
      }.toString()
    },
    by_date: {
      map: function (doc) {
        let date
        if (doc.user_date) {
          date = doc.user_date
        } else if (doc.original_date) {
          date = doc.original_date
        } else {
          date = doc.import_date
        }
        const location = 'location' in doc ? doc.location : null
        emit(date, [date, doc.filename, location, doc.mimetype])
      }.toString()
    },
    by_filename: {
      map: function (doc) {
        let date
        if (doc.user_date) {
          date = doc.user_date
        } else if (doc.original_date) {
          date = doc.original_date
        } else {
          date = doc.import_date
        }
        const location = 'location' in doc ? doc.location : null
        emit(doc.filename.toLowerCase(), [date, doc.filename, location, doc.mimetype])
      }.toString()
    },
    by_location: {
      map: function (doc) {
        if (doc.location) {
          let date
          if (doc.user_date) {
            date = doc.user_date
          } else if (doc.original_date) {
            date = doc.original_date
          } else {
            date = doc.import_date
          }
          emit(doc.location.toLowerCase(), [date, doc.filename, doc.location, doc.mimetype])
        }
      }.toString()
    },
    by_mimetype: {
      map: function (doc) {
        let date
        if (doc.user_date) {
          date = doc.user_date
        } else if (doc.original_date) {
          date = doc.original_date
        } else {
          date = doc.import_date
        }
        const location = 'location' in doc ? doc.location : null
        emit(doc.mimetype.toLowerCase(), [date, doc.filename, location, doc.mimetype])
      }.toString()
    },
    by_tag: {
      map: function (doc) {
        if (doc.tags && Array.isArray(doc.tags)) {
          let date
          if (doc.user_date) {
            date = doc.user_date
          } else if (doc.original_date) {
            date = doc.original_date
          } else {
            date = doc.import_date
          }
          const location = 'location' in doc ? doc.location : null
          doc.tags.forEach(function (tag) {
            emit(tag.toLowerCase(), [date, doc.filename, location, doc.mimetype])
          })
        }
      }.toString()
    },
    all_locations: {
      map: function (doc) {
        if (doc.location) {
          emit(doc.location.toLowerCase(), 1)
        }
      }.toString(),
      reduce: '_count'
    },
    all_tags: {
      map: function (doc) {
        if (doc.tags && Array.isArray(doc.tags)) {
          doc.tags.forEach(function (tag) {
            emit(tag.toLowerCase(), 1)
          })
        }
      }.toString(),
      reduce: '_count'
    },
    all_years: {
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
    const oldDoc = await db.get(index._id)
    if (oldDoc.version === undefined || oldDoc.version < index.version) {
      const ok = await migrate.migrate(db, oldDoc.version || 0, index.version)
      if (ok) {
        await db.put({ ...index, _rev: oldDoc._rev })
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
  // clean up any stale indices from previous versions
  await db.viewCleanup()
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
  const promises = []
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
  const indexCreated = await createIndices(assetsDefinition)
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
  await db.destroy()
  db = new PouchDB(dbPath)
  return initDatabase()
}

/**
 * Test-only function for testing migration.
 */
function getDbObject () {
  return db
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
    const oldDoc = await db.get(newDoc._id)
    await db.put({ ...newDoc, _rev: oldDoc._rev })
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
    const action = res ? 'updated existing' : 'inserted new'
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
 * Look for the asset by the given checksum, returning the identifier, or
 * null if not found.
 *
 * @param {string} checksum - checksum by which to find asset identifier.
 * @returns {Promise<string>} Promise resolving to asset identifier.
 */
function byChecksum (checksum) {
  // should only be 1 result, but limit to 1 anyway
  return db.query('assets/by_checksum', {
    key: lowerStr(checksum),
    limit: 1
  }).then(function (res) {
    return res.rows.length ? res.rows[0].id : null
  }).catch(function (err) {
    logger.warn('byChecksum error:', err)
    return null
  })
}

/**
 * Retrieves all of the tags, as an array of objects.
 *
 * @returns {Promise<Array>} Promise resolving to array of tag objects.
 */
function allTags () {
  return db.query('assets/all_tags', {
    group_level: 1
  }).then(function (res) {
    return res.rows
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
  return db.query('assets/all_locations', {
    group_level: 1
  }).then(function (res) {
    return res.rows
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
  return db.query('assets/all_years', {
    group_level: 1
  }).then(function (res) {
    return res.rows
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
  const allDocs = await db.allDocs()
  // Count those documents that have id starting with "_design/" then subtract
  // that from the total_rows to find the true asset count.
  const designCount = allDocs.rows.reduce((acc, row) => {
    return row.id.startsWith('_design/') ? acc + 1 : acc
  }, 0)
  return allDocs.total_rows - designCount
}

// Fields selected by mango queries to replicate the behavior of the map/reduce
// queries, which return a particular set of fields.
const queryFields = [
  '_id',
  'user_date',
  'original_date',
  'import_date',
  'filename',
  'location',
  'mimetype'
]

/**
 * Massage the mango query results into an object populated with default
 * values for missing data, and the preferred date value.
 *
 * @param {Object} fields - result of a mango query, as in queryFields.
 * @return {Array} result as an object.
 */
function massageMangoResult (fields) {
  return {
    id: fields._id,
    datetime: getBestDate(fields),
    filename: fields.filename,
    location: fields.location || null,
    mimetype: fields.mimetype
  }
}

/**
 * Massage the map/reduce results into an object whose shape matches that
 * of the massageMangoResult() function.
 *
 * @param {Object} fields - result of a mango query, as in queryFields.
 * @return {Array} result as an object.
 */
function massageMapResult (result) {
  return {
    id: result.id,
    datetime: result.value[0],
    filename: result.value[1],
    location: result.value[2],
    mimetype: result.value[3]
  }
}

/**
 * Find assets whose best date falls within the given range.
 *
 * Either after or before may be null, but not both.
 *
 * @param {Number} after - date of oldest asset to include in results.
 * @param {Number} before - date of newest asset to include in results.
 * @return {Array} promise resolving to a list of result objects.
 */
async function findByDateRange (after, before) {
  const keys = after ? (before ? {
    start_key: after,
    end_key: before
  } : {
    start_key: after
  }) : {
    end_key: before
  }
  const queryResults = await db.query('assets/by_date', keys)
  return queryResults.rows.map((row) => massageMapResult(row))
}

/**
 * Find assets whose tag field contains all of the given values.
 *
 * If the tags argument is a list containing a single null element, this
 * function will find all assets that have no tags at all.
 *
 * @param {String} tags - values to look for in the tags field.
 * @return {Array} promise resolving to a list of result objects.
 */
async function findByTags (tags) {
  if (tags.length === 1 && tags[0] === null) {
    // special case to find assets with no tags at all
    // (this special selector scans all documents)
    const queryResults = await db.find({
      selector: {
        $or: [
          { tags: { $exists: false } },
          { tags: { $type: 'null' } },
          { tags: { $size: 0 } }
        ]
      },
      fields: queryFields
    })
    return queryResults.docs.map((fields) => massageMangoResult(fields))
  } else {
    // Use map/reduce for this query, as mango scans all rows.
    const queryResults = await db.query('assets/by_tag', {
      keys: Array.from(tags).sort()
    })
    // Reduce the results to those that have all of the given tags.
    const tagCounts = queryResults.rows.reduce((acc, row) => {
      const docId = row.id
      const count = acc.has(docId) ? acc.get(docId) : 0
      acc.set(docId, count + 1)
      return acc
    }, new Map())
    const matchingRows = queryResults.rows.filter((row) => {
      return tagCounts.get(row.id) === tags.length
    })
    // Remove the duplicate rows by sorting on the document identifier and
    // removing any duplicates.
    const uniqueResults = matchingRows.sort((a, b) => {
      return a.id.localeCompare(b.id)
    }).filter((row, idx, arr) => idx === 0 || row.id !== arr[idx - 1].id)
    return uniqueResults.map((row) => massageMapResult(row))
  }
}

/**
 * Find assets whose location field matches one of those given.
 *
 * If the locations argument is a list containing a single null element, this
 * function will find all assets that have no location at all.
 *
 * @param {String} locations - values to look for in the location field.
 * @return {Array} promise resolving to a list of result objects.
 */
async function findByLocations (locations) {
  if (locations.length === 1 && locations[0] === null) {
    // special case to find assets with no location at all
    // (this special selector scans all documents)
    const queryResults = await db.find({
      selector: {
        $or: [
          { location: { $exists: false } },
          { location: { $type: 'null' } },
          { location: { $eq: '' } }
        ]
      },
      fields: queryFields
    })
    return queryResults.docs.map((fields) => massageMangoResult(fields))
  } else {
    // Use map/reduce for this query, as mango scans all rows.
    const queryResults = await db.query('assets/by_location', {
      keys: Array.from(locations).sort()
    })
    return queryResults.rows.map((row) => massageMapResult(row))
  }
}

/**
 * Find assets whose filename field matches that given.
 *
 * @param {String} filename - value to look for in the filename field.
 * @return {Array} promise resolving to a list of result objects.
 */
async function findByFilename (filename) {
  const queryResults = await db.query('assets/by_filename', {
    key: filename
  })
  return queryResults.rows.map((row) => massageMapResult(row))
}

/**
 * Find assets whose mimetype field matches that given.
 *
 * @param {String} mimetype - value to look for in the mimetype field.
 * @return {Array} promise resolving to a list of result objects.
 */
async function findByMimetype (mimetype) {
  const queryResults = await db.query('assets/by_mimetype', {
    key: mimetype
  })
  return queryResults.rows.map((row) => massageMapResult(row))
}

/**
 * Filter query results by the given date range. If both after and before
 * are null, the rows are returned unfiltered.
 *
 * @param {Number} after - date of oldest asset to include in results.
 * @param {Number} before - date of newest asset to include in results.
 * @param {Array} rows - query results to be filtered.
 * @returns {Array} filtered results.
 */
function filterByDateRange (after, before, rows) {
  if (after && before) {
    return rows.filter((row) => {
      return row.datetime > after && row.datetime < before
    })
  }
  if (after) {
    return rows.filter((row) => {
      return row.datetime > after
    })
  }
  if (before) {
    return rows.filter((row) => {
      return row.datetime < before
    })
  }
  return rows
}

/**
 * Filter query results by the given locations.
 *
 * @param {Number} after - date of oldest asset to include in results.
 * @param {Number} before - date of newest asset to include in results.
 * @param {Array} rows - query results to be filtered.
 * @returns {Array} filtered results.
 */
function filterByLocations (locations, rows) {
  return rows.filter(row => locations.some(loc => loc === lowerStr(row.location)))
}

/**
 * Filter query results by the given filename.
 *
 * @param {Number} filename - value to match to the filename field.
 * @param {Array} rows - query results to be filtered.
 * @returns {Array} filtered results.
 */
function filterByFilename (filename, rows) {
  return rows.filter(row => lowerStr(row.filename) === filename)
}

/**
 * Filter query results by the given mimetype.
 *
 * @param {Number} mimetype - value to match to the mimetype field.
 * @param {Array} rows - query results to be filtered.
 * @returns {Array} filtered results.
 */
function filterByMimetype (mimetype, rows) {
  return rows.filter(row => lowerStr(row.mimetype) === mimetype)
}

/**
 * Search for assets by the given set of parameters.
 *
 * @param {Object} params - query parameters by which to find assets.
 * @param {Array} params.tags - if defined, set of tags which asset must have all of.
 * @param {Array} params.locations - if defined, locations which an asset must have one of.
 * @param {Number} params.after - if defined, UTC millis for which asset date must be greater.
 * @param {Number} params.before - if defined, UTC millis for which asset date must be less.
 * @param {String} params.filename - if defined, asset must have matching filename.
 * @param {String} params.mimetype - if defined, asset must have matching mimetype.
 * @param {Object} params.order - if defined, how to order the results
 * @param {String} params.order.field - name of field by which to order results.
 * @param {String} params.order.dir - order direction (ASC or DESC), default ASC.
 * @returns {Promise<Array>} Promise resolving to list of asset summaries.
 */
async function query (params) {
  let searchBy = null
  // set "search by" according to query params and index precedence;
  // must start with tags, if given, as the other indices do not have
  // the tags field for us to filter on
  const filterParams = lowerCaseParams(params)
  if (filterParams.tags && filterParams.tags.length) {
    searchBy = findByTags.bind(null, filterParams.tags)
    delete filterParams.tags
  } else if (filterParams.after || filterParams.before) {
    searchBy = findByDateRange.bind(null, filterParams.after, filterParams.before)
    delete filterParams.after
    delete filterParams.before
  } else if (filterParams.locations && filterParams.locations.length) {
    searchBy = findByLocations.bind(null, filterParams.locations)
    delete filterParams.locations
  } else if (filterParams.filename) {
    searchBy = findByFilename.bind(null, filterParams.filename)
    delete filterParams.filename
  } else if (filterParams.mimetype) {
    searchBy = findByMimetype.bind(null, filterParams.mimetype)
    delete filterParams.mimetype
  } else {
    // if no parameters are given, then return nothing (for now)
    searchBy = () => { return [] }
  }
  // set "filter by" based on remaining parameters, in precedence order
  // (tags is not even an option, there are none in the query results)
  const filterBy = []
  if (filterParams.after || filterParams.before) {
    filterBy.push(filterByDateRange.bind(null, filterParams.after, filterParams.before))
  }
  if (filterParams.locations && filterParams.locations.length) {
    filterBy.push(filterByLocations.bind(null, filterParams.locations))
  }
  if (filterParams.filename) {
    filterBy.push(filterByFilename.bind(null, filterParams.filename))
  }
  if (filterParams.mimetype) {
    filterBy.push(filterByMimetype.bind(null, filterParams.mimetype))
  }
  // perform the search
  const searchResults = await searchBy()
  // filter the results
  const filteredResults = filterBy.reduce((acc, fn) => fn(acc), searchResults)
  // TODO: sort the results according to the desired 'order'
  // set "order by" based on params
  return filteredResults
}

// Lower case a string, if it is truthy.
const lowerStr = (str) => {
  return str ? str.toLowerCase() : str
}

// Lowercase all of the case insensitive fields, returning a new object.
function lowerCaseParams (params) {
  const lowerList = (lst) => {
    // process non-empty lists whose first element is not null
    if (lst && lst.length > 0 && lst[0] !== null) {
      return lst.map(e => e.toLowerCase())
    }
    return lst
  }
  return Object.assign({}, params, {
    tags: lowerList(params.tags),
    locations: lowerList(params.locations),
    filename: lowerStr(params.filename),
    mimetype: lowerStr(params.mimetype)
  })
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
  for (const field of bestDateOrder) {
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
  byChecksum,
  fetchDocument,
  getBestDate,
  initDatabase,
  reinitDatabase,
  getDbObject,
  query,
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
