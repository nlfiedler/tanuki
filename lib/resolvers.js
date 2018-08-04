//
// Copyright (c) 2018 Nathan Fiedler
//
const _ = require('lodash')
const logger = require('winston')
const backend = require('lib/backend')
const thumbs = require('lib/thumbs')
const {GraphQLScalarType} = require('graphql')
const {Kind} = require('graphql/language')

// Return an integer given the input value. If value is nil, then return
// default. If value is an integer, return that, bounded by the minimum and
// maximum values. If value is a string, parse as an integer and ensure it
// falls within the minimum and maximum bounds.
function boundedIntValue (value, fallback, minimum, maximum) {
  let v = parseInt(value)
  return Math.min(Math.max(isNaN(v) ? fallback : v, minimum), maximum)
}

// Produce an object representing the given asset, suitable for returning from
// the resolver. Defaults are applied, fields renamed, and dates populated.
function prepareAssetResult (asset) {
  const buf = Buffer.from(asset['_id'], 'base64')
  const relpath = buf.toString('utf8')
  return {
    caption: null,
    duration: null,
    location: null,
    ...asset,
    id: asset['_id'],
    filepath: relpath,
    datetime: backend.getBestDate(asset),
    userdate: asset['user_date']
  }
}

// Merge existing asset details with values from a mutation request.
function receiveAssetInput (asset, assetInput) {
  if (_.isArray(assetInput['tags'])) {
    asset.tags = _.sortedUniq(assetInput.tags.sort())
  }
  if (_.isString(assetInput['location'])) {
    // If location is given, overwrite document field.
    asset.location = assetInput.location
  }
  if (_.isString(assetInput['caption'])) {
    asset.caption = assetInput.caption
    // Split the caption on whitespace so we can examine if there are any tags
    // or location(s) that can be copied to the appropriate fields.
    let parts = assetInput.caption.split(/\s+/)
    // Find the #tags and split those on # too, flattening the list and pruning
    // any empty strings (since '#tag'.split('#') yields ['', 'tag']).
    let tags = _.flatMap(parts.filter(w => w.startsWith('#')), e => e.split('#')).filter(i => i)
    // Merge the existing tags with the incoming set. Any old tags that match
    // the new tags in a case insensitive manner will be removed (allowing the
    // case of the tags to be updated). The result will be sorted and made
    // unique.
    asset.tags = _.uniqBy(tags.concat(asset.tags), s => s.toLowerCase()).sort()
    // First word starting with '@' is treated as location, but only if the
    // document does not already have a location defined.
    if (!asset.location) {
      // Start by looking for parts starting with @" or "@ and find the next
      // part that ends with " and join together to get the desired location.
      const startAt = parts.findIndex(w => w.startsWith('@"') || w.startsWith('"@'))
      if (startAt >= 0) {
        const endAt = parts.slice(startAt).findIndex(w => w.endsWith('"'))
        if (endAt >= 0) {
          const location = parts.slice(startAt, startAt + endAt + 1).join(' ')
          asset.location = location.substr(2, location.length - 3)
        }
      } else {
        // If that didn't work, just look for one word starting with @ alone.
        let location = parts.filter(w => w.startsWith('@')).shift()
        if (location) {
          asset.location = location.substr(1)
        }
      }
    }
  }
  if (_.isNumber(assetInput['datetime'])) {
    asset.user_date = assetInput.datetime
  } else if (_.isNull(assetInput['datetime'])) {
    asset.user_date = null
  }
  if (_.isString(assetInput['mimetype'])) {
    asset.mimetype = assetInput.mimetype.toLowerCase()
  }
  return asset
}

const DateTime = new GraphQLScalarType({
  name: 'Date',
  description: (
    'Date/time custom scalar type, represented as the ' +
    'number of milliseconds since 1 January 1970 UTC.'
  ),
  parseValue (value) {
    // we receive and store the UTC milliseconds as-is
    return value
  },
  serialize (value) {
    // we send the UTC milliseconds as-is
    return value
  },
  parseLiteral (ast) {
    if (ast.kind === Kind.INT) {
      return parseInt(ast.value, 10)
    }
    return null
  }
})

const Query = {
  async asset (obj, args, context, info) {
    try {
      const asset = await backend.fetchDocument(args.id)
      return prepareAssetResult(asset)
    } catch (err) {
      if (err.status !== 404) {
        logger.error('Query.asset error:', err.message)
      }
      return null
    }
  },

  count (obj, args, context, info) {
    return backend.assetCount()
  },

  async lookup (obj, args, context, info) {
    if (!args.checksum.includes('-')) {
      throw new Error('checksum must have algorithm prefix')
    }
    const assetId = await backend.byChecksum(args.checksum)
    if (assetId) {
      const asset = await backend.fetchDocument(assetId)
      return prepareAssetResult(asset)
    }
    return null
  },

  async search (obj, args, context, info) {
    let rows = await backend.query(args.params)
    // sort by date by default
    rows.sort((a, b) => b['datetime'] - a['datetime'])
    const totalCount = rows.length
    const count = boundedIntValue(args.count, 10, 1, 10000)
    const offset = boundedIntValue(args.offset, 0, 0, totalCount)
    let pageRows = rows.slice(offset, offset + count)
    // decorate the results with information about the thumbnails
    let thumbRows = []
    for (let elem of pageRows) {
      const dims = await thumbs.getSize(elem.id)
      thumbRows.push({
        ...elem,
        thumbWidth: dims.width,
        thumbHeight: dims.height
      })
    }
    return {
      results: thumbRows,
      count: totalCount
    }
  },

  async locations (obj, args, context, info) {
    let locations = await backend.allLocations()
    // convert the field names to match the schema
    return locations.map(v => ({value: v.key, count: v.value}))
  },

  async tags (obj, args, context, info) {
    let tags = await backend.allTags()
    // convert the field names to match the schema
    return tags.map(v => ({value: v.key, count: v.value}))
  },

  async years (obj, args, context, info) {
    let years = await backend.allYears()
    // convert the field names to match the schema
    return years.map(v => ({value: v.key, count: v.value}))
  }
}

const Mutation = {
  async update (obj, args, context, info) {
    const asset = await backend.fetchDocument(args.id)
    // merge the new values into the existing document
    const updated = receiveAssetInput(asset, args.asset)
    await backend.updateDocument(updated)
    return prepareAssetResult(updated)
  }
}

module.exports = {
  Date: DateTime,
  Query,
  Mutation
}
