//
// Copyright (c) 2017 Nathan Fiedler
//
const config = require('config')
const fs = require('fs-extra')
const path = require('path')
const logger = require('winston')
const sharp = require('sharp')
const ffmpeg = require('fluent-ffmpeg')
const LRU = require('lru-cache')

//
// Code for operating on the stored assets.
//

const assetsPath = config.get('backend.assetPath')

const thumbnailCache = LRU({
  max: 10485760,
  length: function (n, key) { return n.length }
})

/**
 * Convert the SHA256 checksum to the full path of the asset.
 *
 * @returns {string} absolute path of the asset.
 */
function assetPath (checksum) {
  const part1 = checksum.slice(0, 2)
  const part2 = checksum.slice(2, 4)
  const part3 = checksum.slice(4)
  let asp = path.join(assetsPath, part1, part2, part3)
  return path.isAbsolute(asp) ? asp : path.join(process.cwd(), asp)
}

/**
 * Retrieve the thumbnail for the asset identified by the given checksum.
 * The image will be bounded to a box 240 by 240 pixels.
 *
 * @param {string} mimetype - the mimetype of the file.
 * @param {string} checksum - SHA256 digest of the file.
 * @returns {Promise<Buffer>} promise resolving to an object holding
 *          the nodejs Buffer ('binary') and mime type ('mimetype');
 *          resolves to null if no thumbnail available.
 */
async function retrieveThumbnail (mimetype, checksum) {
  const cached = thumbnailCache.get(checksum)
  if (cached) {
    logger.info(`cache hit for ${checksum}`)
    return cached
  }
  logger.info(`cache miss for ${checksum}`)
  const filepath = assetPath(checksum)
  if (!fs.existsSync(filepath)) {
    logger.warn(`asset not found for ${checksum}`)
    return null
  }
  const thumbnail = await generateThumbnail(mimetype, filepath, 240)
  if (thumbnail) {
    // only cache valid objects
    logger.debug(`cache store for ${checksum}`)
    thumbnailCache.set(checksum, thumbnail)
  }
  return thumbnail
}

// Generate a (smaller) image for the asset. May return null.
async function generateThumbnail (mimetype, filepath, pixels) {
  if (mimetype.startsWith('video/')) {
    return generateVideoThumbnail(filepath, pixels)
  } else if (mimetype.startsWith('image/')) {
    // fit the image into a box of the given size, convert to jpeg
    let bits = await sharp(filepath)
      .resize(pixels, pixels)
      .max()
      .withoutEnlargement()
      .toFormat('jpeg')
      .toBuffer()
    return {
      binary: bits,
      mimetype: 'image/jpg'
    }
  } else {
    return null
  }
}

/**
 * Generate a thumbnail for the video asset.
 *
 * @param {string} filepath - path of the video asset.
 * @param {number} pixels - size of the desired image.
 * @returns {Promise<Number>} promise resolving to an object holding
 *          the nodejs Buffer ('binary') and mime type ('mimetype');
 *          resolves to null if asset is missing.
 */
function generateVideoThumbnail (filepath, pixels) {
  // output file will live alongside the video asset, permanently
  const filename = `${filepath}.jpg`
  // if it already exists, return it
  if (fs.existsSync(filename)) {
    let binary = fs.readFileSync(filename)
    return Promise.resolve({
      binary,
      mimetype: 'image/jpg'
    })
  }
  // otherwise generate it now
  return new Promise((resolve, reject) => {
    ffmpeg(filepath)
      .on('end', function () {
        let binary = fs.readFileSync(filename)
        resolve({
          binary,
          mimetype: 'image/jpg'
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
        `scale=w=${pixels}:h=${pixels}:force_original_aspect_ratio=decrease`
      ])
      .save(filename)
  })
}

/**
 * Generate a "preview" sized version of the asset by the given checksum.
 * The image will be bounded to a box 640 by 640 pixels.
 *
 * @param {string} mimetype - the mimetype of the file.
 * @param {string} checksum - SHA256 digest of the file.
 * @returns {Promise<Buffer>} promise resolving to an object holding
 *          the nodejs Buffer ('binary') and mime type ('mimetype');
 *          resolves to null if asset is missing.
 */
async function generatePreview (mimetype, checksum) {
  let filepath = assetPath(checksum)
  if (!fs.existsSync(filepath)) {
    return null
  }
  return generateThumbnail(mimetype, filepath, 640)
}

/**
 * Retrieve the duration of the asset, if it is a video, or null if not.
 *
 * @param {string} mimetype - the mimetype of the file.
 * @param {string} checksum - SHA256 digest of the file.
 * @returns {Promise<Number>} promise resolving to the duration in seconds,
 *          or null if unable to extract data.
 */
async function getDuration (mimetype, checksum) {
  let filepath = assetPath(checksum)
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
            for (let stream of metadata.streams) {
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
  getDuration,
  retrieveThumbnail
}
