//
// Copyright (c) 2018 Nathan Fiedler
//
const config = require('config')
const fs = require('fs-extra')
const logger = require('winston')
const PouchDB = require('pouchdb')
const sharp = require('sharp')
const assets = require('lib/assets')
const backend = require('lib/backend')

//
// Code for caching "wide" asset thumbnail metadata.
//

const dbPath = config.get('backend.thumbsDbPath')
fs.ensureDirSync(dbPath)
const db = new PouchDB(dbPath)

// Compute the size of the asset thumbnail and cache the result.
async function computeSize (assetId) {
  try {
    let doc = await backend.fetchDocument(assetId)
    let mimetype = doc.mimetype ? doc.mimetype : 'application/octet-stream'
    let resized = await assets.generateWideThumb(mimetype, assetId)
    let cached = {
      _id: assetId
    }
    if (resized) {
      const metadata = await sharp(resized.binary).metadata()
      cached.width = metadata.width
      cached.height = metadata.height
    } else {
      // remember the fact that this asset does not have a thumbnail
      logger.info(`computeSize: no thumbnail for ${assetId}`)
      cached.width = -1
      cached.height = -1
    }
    await db.put(cached)
    return cached
  } catch (err) {
    if (err.status === 404) {
      logger.warn(`computeSize: no such asset ${assetId}`)
      return {
        _id: assetId,
        width: 0,
        height: 0
      }
    } else if (err.status === 409) {
      // The Elm client is seemingly making the same request twice, before the
      // first has had a chance to respond. As such, conflicts will be common
      // until the database is populated.
      logger.warn(`computeSize: update conflict for ${assetId}`)
      return db.get(assetId)
    } else {
      throw err
    }
  }
}

/**
 * Retrieve the size of the "wide" thumbnail for the given asset, computing
 * as necessary, and caching for efficiency.
 */
async function getSize (assetId) {
  try {
    // need the await in order for try/catch to work
    const doc = await db.get(assetId)
    logger.info(`getSize: cache hit for ${assetId}`)
    return doc
  } catch (err) {
    if (err.status === 404) {
      logger.info(`getSize: cache miss for ${assetId}`)
      return computeSize(assetId)
    } else {
      throw err
    }
  }
}

module.exports = {
  getSize
}
