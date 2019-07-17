//
// Copyright (c) 2018 Nathan Fiedler
//
const config = require('config')
const dateFormat = require('dateformat')
const fs = require('fs-extra')
const path = require('path')
const logger = require('logging')
const sharp = require('sharp')
const ffmpeg = require('fluent-ffmpeg')
const ffmpegStatic = require('ffmpeg-static')
ffmpeg.setFfmpegPath(ffmpegStatic.path)
const ffmpegProbe = require('ffprobe-static')
ffmpeg.setFfprobePath(ffmpegProbe.path)
const LRU = require('lru-cache')
const ULID = require('ulid')

//
// Code for operating on the stored assets.
//

const assetsPath = config.get('backend.assetPath')

const thumbnailCache = new LRU({
  max: 10485760,
  length: function (n, key) { return n.length }
})

/**
 * Convert the asset identifier to the file path of the asset.
 *
 * @param {string} assetId - asset identifier.
 * @returns {string} absolute path of the asset file.
 */
function assetPath (assetId) {
  const buf = Buffer.from(assetId, 'base64')
  const relpath = buf.toString('utf8')
  const fp = path.join(assetsPath, relpath)
  return path.isAbsolute(fp) ? fp : path.join(process.cwd(), fp)
}

/**
 * Convert the filename into a relative path where the asset will be located in
 * storage, and return as a base64 encoded value, suitable as an identifier.
 *
 * This is _not_ a pure function, since it involves both the current time as
 * will as a random number. It does, however, avoid any possibility of name
 * collisions.
 *
 * @param {Object} datetime - date/time of asset import, either a Date or a Number.
 * @param {string} filename - original name of the asset.
 * @returns {string} relative path of new asset, base64 encoded.
 */
function makeAssetId (datetime, filename) {
  // round the date/time down to the nearest quarter hour
  // (e.g. 21:50 becomes 21:45, 08:10 becomes 08:00)
  // this requires having a real Date object
  if (Number.isInteger(datetime)) {
    datetime = new Date(datetime)
  }
  const roundate = new Date(
    datetime.getFullYear(),
    datetime.getMonth(),
    datetime.getDate(),
    datetime.getHours(),
    Math.floor(datetime.getMinutes() / 15) * 15
  )
  const leadingPath = dateFormat(roundate, 'yyyy/mm/dd/HHMM')
  const name = ULID.ulid() + path.extname(filename)
  const relpath = path.join(leadingPath, name).toLowerCase()
  const buf = Buffer.from(relpath, 'utf8')
  return buf.toString('base64')
}

/**
 * Retrieve the thumbnail for the identified asset.
 * The image will be bounded to a box 240 by 240 pixels.
 *
 * @param {string} mimetype - the mimetype of the file.
 * @param {string} assetId - asset identifier.
 * @returns {Promise<Buffer>} promise resolving to an object holding
 *          the nodejs Buffer ('binary') and mime type ('mimetype');
 *          resolves to null if no thumbnail available.
 */
async function retrieveThumbnail (mimetype, assetId) {
  const cached = thumbnailCache.get(assetId)
  if (cached) {
    logger.info(`cache hit for ${assetId}`)
    return cached
  }
  logger.info(`cache miss for ${assetId}`)
  const filepath = assetPath(assetId)
  if (!fs.existsSync(filepath)) {
    logger.warn(`asset not found for ${assetId}`)
    return null
  }
  const thumbnail = await generateThumbnail(mimetype, filepath, 240, 240)
  if (thumbnail) {
    // only cache valid objects
    logger.debug(`cache store for ${assetId}`)
    thumbnailCache.set(assetId, thumbnail)
  }
  return thumbnail
}

/**
 * Generate a resized image for the given visual asset.
 *
 * @param {string} mimetype - the mimetype of the file.
 * @param {string} filepath - path of the file.
 * @param {number} width - width in pixels of the desired image.
 * @param {number} height - height in pixels of the desired image.
 * @returns {Promise<Buffer>} promise resolving to an object holding
 *          the nodejs Buffer ('binary') and mime type ('mimetype');
 *          resolves to null if no thumbnail available.
 */
async function generateThumbnail (mimetype, filepath, width, height) {
  if (width === null && height === null) {
    throw new Error('cannot have null width and height')
  }
  if (mimetype.startsWith('video/')) {
    return generateVideoThumbnail(filepath, width, height)
  } else if (mimetype.startsWith('image/')) {
    // fit the image into a box of the given size, convert to jpeg
    const bits = await sharp(filepath)
      .resize({
        width,
        height,
        fit: 'inside',
        withoutEnlargement: true
      })
      .toFormat('jpeg')
      .toBuffer()
    return {
      binary: bits,
      mimetype: 'image/jpeg'
    }
  } else {
    return null
  }
}

/**
 * Generate a thumbnail for the video asset.
 *
 * @param {string} filepath - path of the video asset.
 * @param {number} width - width in pixels of the desired image.
 * @param {number} height - height in pixels of the desired image.
 * @returns {Promise<Number>} promise resolving to an object holding
 *          the nodejs Buffer ('binary') and mime type ('mimetype');
 *          resolves to null if asset is missing.
 */
function generateVideoThumbnail (filepath, width, height) {
  // output file will remain on disk indefinitely, and be overwritten each
  // time, since the pixels value could be different; no sense keeping a
  // file for every possible pixels value either
  const dirname = path.dirname(filepath)
  const basename = path.basename(filepath, path.extname(filepath))
  const outfile = path.join(dirname, basename + '.jpg')
  let scale
  if (width && height) {
    scale = `w=${width}:h=${height}:force_original_aspect_ratio=decrease`
  } else if (width) {
    scale = `w=${width}:force_original_aspect_ratio=decrease`
  } else if (height) {
    scale = `h=${height}:force_original_aspect_ratio=decrease`
  }
  return new Promise((resolve, reject) => {
    ffmpeg(filepath)
      .on('end', function () {
        const binary = fs.readFileSync(outfile)
        resolve({
          binary,
          mimetype: 'image/jpeg'
        })
      })
      .on('error', function (err) {
        reject(err)
      })
      // There is the tempting screenshots() function, but unfortunately it does
      // not offer a means of maintaining the aspect ratio, and it insists on
      // writing the output as huge PNG files.
      //
      // see https://github.com/fluent-ffmpeg/node-fluent-ffmpeg/issues/776
      .outputOptions([
        '-vframes',
        '1',
        '-an',
        '-filter:v',
        `scale=${scale}`
      ])
      .save(outfile)
  })
}

/**
 * Generate a thumbnail image to fit a height of 300 pixels. This leaves the
 * width of the image to be determined by the original aspect ratio.
 *
 * @param {string} mimetype - the mimetype of the file.
 * @param {string} assetId - asset identifier.
 * @returns {Promise<Buffer>} promise resolving to an object holding
 *          the nodejs Buffer ('binary') and mime type ('mimetype');
 *          resolves to null if no thumbnail available.
 */
async function generateWideThumb (mimetype, assetId) {
  const filepath = assetPath(assetId)
  if (!fs.existsSync(filepath)) {
    logger.warn(`asset not found for ${assetId}`)
    return null
  }
  if (mimetype.startsWith('video/')) {
    return generateVideoThumbnail(filepath, null, 300)
  } else if (mimetype.startsWith('image/')) {
    // resize the image to the desired height, convert to jpeg
    const bits = await sharp(filepath)
      .resize(null, 300)
      .toFormat('jpeg')
      .toBuffer()
    return {
      binary: bits,
      mimetype: 'image/jpeg'
    }
  } else {
    return null
  }
}

/**
 * Generate a "preview" sized version of the identified asset.
 * The image will be bounded to a box 640 by 640 pixels.
 *
 * @param {string} mimetype - the mimetype of the file.
 * @param {string} assetId - asset identifier.
 * @returns {Promise<Buffer>} promise resolving to an object holding
 *          the nodejs Buffer ('binary') and mime type ('mimetype');
 *          resolves to null if asset is missing.
 */
async function generatePreview (mimetype, assetId) {
  const filepath = assetPath(assetId)
  if (!fs.existsSync(filepath)) {
    return null
  }
  return generateThumbnail(mimetype, filepath, 640, 640)
}

/**
 * Retrieve the duration of the asset, if it is a video, or null if not.
 *
 * @param {string} mimetype - the mimetype of the file.
 * @param {string} filepath - full path to the file.
 * @returns {Promise<Number>} promise resolving to the duration in seconds,
 *          or null if unable to extract data.
 */
async function getDuration (mimetype, filepath) {
  if (!fs.existsSync(filepath)) {
    return Promise.resolve(null)
  }
  if (mimetype.startsWith('video/')) {
    return new Promise((resolve, reject) => {
      ffmpeg.ffprobe(filepath, function (err, metadata) {
        if (err) {
          reject(err)
        } else {
          // streams: [{ codec_type: 'video', duration: '2.03'... }]
          try {
            for (const stream of metadata.streams) {
              if (stream.codec_type === 'video') {
                resolve(stream.duration)
              }
            }
            resolve(null)
          } catch (err) {
            resolve(null)
          }
        }
      })
    })
  }
  return Promise.resolve(null)
}

module.exports = {
  assetPath,
  generatePreview,
  generateWideThumb,
  getDuration,
  makeAssetId,
  retrieveThumbnail
}
