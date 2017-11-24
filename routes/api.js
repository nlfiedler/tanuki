//
// Copyright (c) 2017 Nathan Fiedler
//
const express = require('express')
const bodyParser = require('body-parser')
const multer = require('multer')
const config = require('config')
const backend = require('lib/backend')
const incoming = require('lib/incoming')
const router = express.Router()

const uploadPath = config.get('backend.uploadPath')
const upload = multer({ dest: uploadPath })

router.use(bodyParser.json())

// wrapper that directs errors to the appropriate handler
let wrap = fn => (...args) => fn(...args).catch(args[2])

router.use(function (req, res, next) {
  // Disable automatic caching in express. See pending pull request:
  // https://github.com/expressjs/express/pull/2841
  req.headers['if-none-match'] = 'no-match-for-this'
  next()
})

// Ensure the years parameter can be parsed as integers.
router.use(function (req, res, next) {
  if (req.query['years'] !== undefined) {
    let years = req.query['years']
    if (!Array.isArray(years)) {
      res.status(400).send('years must be an array')
    } else {
      let intYears = years.map(y => parseInt(y))
      if (intYears.some(y => isNaN(y))) {
        res.status(400).send('years must be integers')
      } else {
        req.queryYears = intYears
        next()
      }
    }
  } else {
    next()
  }
})

router.get('/assets', wrap(async function (req, res, next) {
  // TODO: param: order (for sorting results)
  let tags = req.query['tags']
  let years = req.queryYears
  let locations = req.query['locations']
  if (!tags && !years && !locations) {
    // when no params are given, return count of assets
    const count = await backend.assetCount()
    res.json({
      assets: [],
      count
    })
  } else if (tags && !Array.isArray(tags)) {
    res.status(400).send('tags must be an array')
  } else if (locations && !Array.isArray(locations)) {
    res.status(400).send('locations must be an array')
  } else {
    let rows = await backend.query(tags, years, locations)
    // Perform some default sorting, with newer assets appearing earlier in the
    // list of results.
    rows.sort((a, b) => b['date'] - a['date'])
    // count is the number of _all_ matching results
    const count = rows.length
    // handle pagination with certain defaults and bounds
    const pageSize = boundedIntValue(req.query['page_size'], 10, 1, 100)
    const pageLimit = Math.ceil(count / pageSize)
    const page = boundedIntValue(req.query['page'], 1, 1, pageLimit)
    const start = (page - 1) * pageSize
    let pageRows = rows.slice(start, start + pageSize)
    let formattedRows = pageRows.map((row) => {
      // the summary includes only the formatted date, no time
      let date = new Date(row['date']).toLocaleDateString()
      return {...row, date}
    })
    res.json({
      assets: formattedRows,
      // include total count of all matching rows
      count
    })
  }
}))

router.post('/assets', upload.single('asset'), wrap(async function (req, res, next) {
  let checksum = await incoming.computeChecksum(req.file.path)
  try {
    // check if an asset with this checksum already exists
    await backend.fetchDocument(checksum)
    res.json({status: 'success', id: checksum})
  } catch (err) {
    if (err.status === 404) {
      let originalDate = incoming.getOriginalDate(req.file.path)
      let importDate = incoming.dateToList(new Date())
      let doc = {
        _id: checksum,
        file_name: req.file.originalname,
        file_size: req.file.size,
        import_date: importDate,
        mimetype: req.file.mimetype,
        original_date: originalDate,
        // everything generally assumes the tags field is not undefined
        tags: []
      }
      await backend.updateDocument(doc)
      incoming.storeAsset(req.file.path, checksum)
      res.json({status: 'success', id: checksum})
    } else {
      // some other error occurred
      res.status(err.status).send(err.message)
    }
  }
}))

router.get('/assets/:id', wrap(async function (req, res, next) {
  try {
    let asset = await backend.fetchDocument(req.params['id'])
    let defaults = {
      caption: null,
      location: null
    }
    res.json({
      ...defaults,
      ...asset,
      checksum: asset['_id'],
      datetime: dateListToString(getBestDate(asset)),
      duration: getDuration(asset),
      user_date: dateListToString(asset['user_date'])
    })
  } catch (err) {
    // PouchDB errors are basically HTTP errors and work fine here.
    res.status(err.status).send(err.message)
  }
}))

router.put('/assets/:id', wrap(async function (req, res, next) {
  try {
    let asset = await backend.fetchDocument(req.params['id'])
    // merge the new values into the existing document
    let updated = {
      ...asset,
      ...req.body
    }
    // perform special field value handling
    if ('tags' in req.body) {
      updated.tags = tagStringToList(req.body.tags)
    }
    if ('user_date' in req.body) {
      if (req.body.user_date && req.body.user_date.length > 0) {
        // pass the original asset for getting the best date, otherwise
        // you get the 'user_date', which we are trying to set right now
        updated.user_date = mergeUserDateWithBest(req.body.user_date, asset)
      } else {
        // wipe out the user date field if no value is given
        updated.user_date = null
      }
    }
    await backend.updateDocument(updated)
    res.json({status: 'success'})
  } catch (err) {
    // PouchDB errors are basically HTTP errors and work fine here.
    res.status(err.status).send(err.message)
  }
}))

router.get('/tags', wrap(async function (req, res, next) {
  let tags = await backend.allTags()
  // convert the field names to something sensible
  let renamed = tags.map((v) => {
    return {tag: v.key, count: v.value}
  })
  res.json(renamed)
}))

router.get('/locations', wrap(async function (req, res, next) {
  let locations = await backend.allLocations()
  // convert the field names to something sensible
  let renamed = locations.map((v) => {
    return {location: v.key, count: v.value}
  })
  res.json(renamed)
}))

router.get('/years', wrap(async function (req, res, next) {
  let years = await backend.allYears()
  // convert the field names to something sensible
  let renamed = years.map((v) => {
    return {year: v.key, count: v.value}
  })
  res.json(renamed)
}))

// Return an integer given the input value. If value is nil, then return
// default. If value is an integer, return that, bounded by the minimum and
// maximum values. If value is a string, parse as an integer and ensure it
// falls within the minimum and maximum bounds.
function boundedIntValue (value, fallback, minimum, maximum) {
  let v = parseInt(value)
  return Math.min(Math.max(isNaN(v) ? fallback : v, minimum), maximum)
}

// Convert the array of integers to a formatted date/time string.
function dateListToString (dl) {
  if (!dl) {
    return ''
  }
  // need to adjust the month for Date object
  return new Date(dl[0], dl[1] - 1, dl[2], dl[3], dl[4]).toLocaleString()
}

// Parse the user date (e.g. '2003-08-30') into an array of numbers, merging the
// time from the best available date in the asset.
function mergeUserDateWithBest (userDate, doc) {
  let [y, m, d] = userDate.split('-').map((x) => parseInt(x))
  let bestDate = getBestDate(doc)
  if (bestDate) {
    return [y, m, d, bestDate[3], bestDate[4]]
  }
  return [y, m, d, 0, 0]
}

// Convert the string of comma-separated tags into an array of sorted and unique
// values.
function tagStringToList (tags) {
  let list = tags.split(',').map((t) => t.trim())
  let uniq = list.sort().filter((t, i, a) => i === 0 || t !== a[i - 1])
  return uniq
}

function getDuration (asset) {
  let mimetype = asset.mimetype ? asset.mimetype : ''
  if (mimetype.startsWith('video/')) {
    // TODO: get the duration of the video file
    // filepath = TanukiBackend.checksum_to_asset_path(checksum)
    // ffprobe_args = [
    //   "-loglevel", "quiet", "-show_entries",
    //   "format=duration", "-of", "default=noprint_wrappers=1:nokey=1",
    //   filepath
    // ]
    // case System.cmd("ffprobe", ffprobe_args) do
    //   {output, 0} ->
    //     round(String.to_float(String.trim(output)))
    //   {output, code} ->
    //     Logger.warn("ffprobe exited non-zero (#{code}): #{output}")
    //     nil
    // end
    return null
  }
  return null
}

// The field names of the date/time values in their preferred order. That is,
// the user-provided value is considered the best, with the Exif original being
// second, and so on.
const bestDateOrder = [
  'user_date',
  'original_date',
  'file_date',
  'import_date'
]

// Retrieve the preferred date/time value from the document.
function getBestDate (doc) {
  for (let field of bestDateOrder) {
    if (field in doc && doc.field) {
      return doc[field]
    }
  }
  return null
}

module.exports = router
