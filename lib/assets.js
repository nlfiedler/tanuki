//
// Copyright (c) 2017 Nathan Fiedler
//
const config = require('config')
const fs = require('fs-extra')
const path = require('path')
// const logger = require('winston')
const sharp = require('sharp')
const ffmpeg = require('fluent-ffmpeg')

//
// Code for operating on the stored assets.
//

const assetsPath = config.get('backend.assetPath')

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
 * @returns {Promise<Buffer>} promise resolving to an object holding
 *          the nodejs Buffer ('binary') and mime type ('mimetype');
 *          resolves to null if asset is missing.
 */
async function retrieveThumbnail (checksum) {
  // TODO: use catbox to cache the thumbnails
  // TODO: handle thumbnails for video files
  let path = assetPath(checksum)
  if (!fs.existsSync(path)) {
    return null
  }
  // fit the image into a box 240 by 240 pixels, convert to jpeg
  let bits = await sharp(path)
    .resize(240, 240)
    .max()
    .withoutEnlargement()
    .toFormat('jpeg')
    .toBuffer()
  return {
    binary: bits,
    mimetype: 'image/jpg'
  }
}

/**
 * Generate a "preview" sized version of the asset by the given checksum.
 * The image will be bounded to a box 640 by 640 pixels.
 *
 * @returns {Promise<Buffer>} promise resolving to an object holding
 *          the nodejs Buffer ('binary') and mime type ('mimetype');
 *          resolves to null if asset is missing.
 */
async function generatePreview (checksum) {
  let path = assetPath(checksum)
  if (!fs.existsSync(path)) {
    return null
  }
  // fit the image into a box 640 by 640 pixels, convert to jpeg
  let bits = await sharp(path)
    .resize(640, 640)
    .max()
    .withoutEnlargement()
    .toFormat('jpeg')
    .toBuffer()
  return {
    binary: bits,
    mimetype: 'image/jpg'
  }
}

/**
 * Retrieve the duration of the asset, if it is a video, or null if not.
 *
 * @returns {Promise<Number>} promise resolving to the duration in seconds,
 *          or null if unable to extract data.
 */
async function getDuration (mimetype, checksum) {
  let path = assetPath(checksum)
  if (!fs.existsSync(path)) {
    return Promise.resolve(null)
  }
  if (mimetype.startsWith('video/')) {
    return new Promise((resolve, reject) => {
      ffmpeg.ffprobe(path, function (err, metadata) {
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
