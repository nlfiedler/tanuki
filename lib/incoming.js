//
// Copyright (c) 2017 Nathan Fiedler
//
const config = require('config')
const fs = require('fs-extra')
const path = require('path')
const logger = require('winston')
// exifreader needs the DataView type (to be overridden)
global.DataView = require('jdataview')
// exifreader also needs DOMParser (to be overridden)
global.DOMParser = require('xmldom').DOMParser
const ExifReader = require('exifreader')
const crypto = require('crypto')
const assets = require('./assets')

//
// Code for incorporating new assets into the system.
//

const assetsPath = config.get('backend.assetPath')
fs.ensureDirSync(assetsPath)

/**
 * Compute the SHA256 digest of the file at the given path.
 *
 * @returns {Promise<string>} Promise resolving to the sha256 digest.
 */
function computeChecksum (filepath) {
  return new Promise((resolve, reject) => {
    const hash = crypto.createHash('sha256')
    const rs = fs.createReadStream(filepath)
    rs.on('error', reject)
    rs.on('data', chunk => hash.update(chunk))
    rs.on('end', () => resolve(hash.digest('hex')))
  })
}

/**
 * Extract the original date/time from the asset. For images that
 * contain Exif data, returns the parsed DateTimeOriginal value.
 *
 * @returns {Array<Number>} the date/time in array format if successful, null otherwise.
 */
function getOriginalDate (filepath) {
  // TODO: check if file is an image before reading
  let image = true
  if (image) {
    let data = fs.readFileSync(filepath)
    const tags = ExifReader.load(data)
    return parseDate(tags['DateTimeOriginal'].description)
  } else {
    // TODO: use ffprobe to get creation time from video assets
    return null
  }
}

function correctOrientation (filepath) {
  // TODO: implement orientation checker
  return true
}

function autoOrient (filepath, destPath) {
  // TODO: implement auto-orientation correction
  fs.copyFileSync(filepath, destPath)
}

/**
 * Convert a Date object to the array of integers expected by the backend.
 *
 * @param {Date} datetime - the date object to convert.
 * @returns {Array<Number>} the date/time in array format.
 */
function dateToList (datetime) {
  return [
    datetime.getFullYear(),
    datetime.getMonth() + 1,
    datetime.getDate(),
    datetime.getHours(),
    datetime.getMinutes()
  ]
}

/**
 * Move the named file to its final destination within the asset store.
 * If the file is an image that requires orientation correction, that
 * action will be performed here.
 *
 * @param {string} filepath - path to incoming file.
 * @param {string} checksum - SHA256 checksum of the file.
 */
function storeAsset (filepath, checksum) {
  // If an existing asset with the same checksum already exists, the new
  // asset will be removed to prevent further processing in the future.
  const destPath = assets.assetPath(checksum)
  if (fs.existsSync(destPath)) {
    logger.info(`ignoring duplicate asset ${filepath} with ${checksum}`)
    fs.unlinkSync(filepath)
  } else {
    fs.ensureDirSync(path.dirname(destPath))
    if (correctOrientation(filepath)) {
      logger.info(`moving ${filepath} to ${destPath}`)
      // use copy to handle crossing file systems
      fs.copyFileSync(filepath, destPath)
    } else {
      autoOrient(filepath, destPath)
      logger.info(`corrected orientation for ${filepath}, saved to ${destPath}`)
    }
    fs.unlinkSync(filepath)
  }
}

const DATE_REGEXP = new RegExp(
  // https://www.media.mit.edu/pia/Research/deepview/exif.html -- DateTime
  //  yyyy  :    MM  :    dd       HH  :    mm  :    ss
  '^(\\d{4}):(\\d{2}):(\\d{2}) (\\d{2}):(\\d{2}):(\\d{2})'
)

// Convert the Exif formatted date/time into an array of numbers
// (e.g. [2003, 09, 03, 17, 24]).
function parseDate (value) {
  const m = DATE_REGEXP.exec(value)
  if (m == null) {
    return null
  }
  return [
    parseInt(m[1]),
    parseInt(m[2]),
    parseInt(m[3]),
    parseInt(m[4]),
    parseInt(m[5])
  ]
}

module.exports = {
  computeChecksum,
  getOriginalDate,
  dateToList,
  storeAsset
}
