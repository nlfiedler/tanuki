//
// Copyright (c) 2017 Nathan Fiedler
//
const config = require('config')
const fs = require('fs-extra')
const path = require('path')
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
async function retrieveThumbnail (mimetype, checksum) {
  // TODO: use catbox to cache the thumbnails
  let path = assetPath(checksum)
  if (!fs.existsSync(path)) {
    return null
  }
  // fit the image into a box 240 by 240 pixels, convert to jpeg
  if (mimetype.startsWith('video/')) {
    return generateVideoThumbnail(checksum)
  } else if (mimetype.startsWith('image/')) {
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
  } else {
    return null
  }
}

/**
 * Generate a thumbnail for the video asset.
 *
 * @returns {Promise<Number>} promise resolving to an object holding
 *          the nodejs Buffer ('binary') and mime type ('mimetype');
 *          resolves to null if asset is missing.
 */
async function generateVideoThumbnail (checksum) {
  const filepath = assetPath(checksum)
  if (!fs.existsSync(filepath)) {
    return Promise.resolve(null)
  }
  // output file will live alongside the video asset
  const filename = `${filepath}.png`
  // if it already exists, return it
  if (fs.existsSync(filename)) {
    let binary = fs.readFileSync(filename)
    return Promise.resolve({
      binary,
      mimetype: 'image/png'
    })
  }
  // otherwise generate it now
  return new Promise((resolve, reject) => {
    ffmpeg(filepath)
      .on('end', function () {
        let binary = fs.readFileSync(filename)
        resolve({
          binary,
          mimetype: 'image/png'
        })
      })
      .on('error', function (err) {
        reject(err)
      })
      //
      // TODO: it is desirable to keep the aspect ratio
      //
      // filed bug: https://github.com/fluent-ffmpeg/node-fluent-ffmpeg/issues/776
      //
      // The options below cannot be combined with -filter_complex, which
      // is apparently what fluent-ffmpeg is using for the screenshot.
      //
      // .outputOptions([
      //   '-filter:v',
      //   'scale=w=240:h=240:force_original_aspect_ratio=decrease'
      // ])
      //
      // For now, specify size as 240x? since most of the time the videos
      // are in landscape mode and thus this will yield the desired image.
      //
      .screenshots({
        count: 1,
        filename: path.basename(filename),
        folder: path.dirname(filename),
        size: '240x?'
      })
  })
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
