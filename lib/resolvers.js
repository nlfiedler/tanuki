//
// Copyright (c) 2018 Nathan Fiedler
//
const assets = require('lib/assets')
const backend = require('lib/backend')
const incoming = require('lib/incoming')

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

// Produce an object representing the given asset, suitable for returning from
// the resolver. Defaults are applied, fields renamed, and dates populated.
function prepareAssetResult (asset) {
  return {
    caption: null,
    duration: null,
    location: null,
    ...asset,
    id: asset['_id'],
    datetime: dateListToString(assets.getBestDate(asset)),
    userdate: dateListToString(asset['user_date'])
  }
}

const Query = {
  async asset (obj, args, context, info) {
    let asset = await backend.fetchDocument(args.id)
    return prepareAssetResult(asset)
  },

  count (obj, args, context, info) {
    return backend.assetCount()
  },

  async search (obj, args, context, info) {
    let rows = await backend.query(args.tags, args.years, args.locations)
    // sort by date by default
    rows.sort((a, b) => b['date'] - a['date'])
    const totalCount = rows.length
    const count = boundedIntValue(args.count, 10, 1, 10000)
    const offset = boundedIntValue(args.offset, 0, 0, totalCount)
    let pageRows = rows.slice(offset, offset + count)
    let formattedRows = pageRows.map(row => (
      {
        id: row.id,
        filename: row.filename,
        // the summary includes only the formatted date, no time
        date: new Date(row.date).toLocaleDateString(),
        location: row.location
      }
    ))
    return {
      results: formattedRows,
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
    const updated = incoming.updateAssetFields(asset, args.asset)
    await backend.updateDocument(updated)
    return prepareAssetResult(updated)
  }
}

module.exports = {
  Query,
  Mutation
}
