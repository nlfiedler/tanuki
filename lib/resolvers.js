//
// Copyright (c) 2018 Nathan Fiedler
//
const _ = require('lodash')
const assets = require('lib/assets')
const backend = require('lib/backend')
const incoming = require('lib/incoming')
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
  return {
    caption: null,
    duration: null,
    location: null,
    ...asset,
    id: asset['_id'],
    datetime: assets.getBestDate(asset),
    userdate: asset['user_date']
  }
}

const DateTime = new GraphQLScalarType({
  name: 'Date',
  description: 'Date/time custom scalar type',
  parseValue (value) {
    // convert UTC milliseconds to date/time int array
    if (_.isNumber(value)) {
      return incoming.dateToList(new Date(value))
    }
    return null
  },
  serialize (value) {
    if (_.isArray(value)) {
      // convert date/time int array to UTC
      let tzMillisOffset = new Date().getTimezoneOffset() * 60000
      return Date.UTC(value[0], value[1] - 1, value[2], value[3], value[4]) + tzMillisOffset
    } else if (_.isNumber(value)) {
      // return search results date/time as-is
      return value
    }
    return null
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
        datetime: row.date,
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
  Date: DateTime,
  Query,
  Mutation
}
