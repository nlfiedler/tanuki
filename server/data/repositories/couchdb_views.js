//
// Copyright (c) 2025 Nathan Fiedler
//

/* global emit */

// JavaScript functions that will be inserted into CouchDB as strings.
//
// Note that they are defined using `export const` instead of `export function`
// in order for the .toString() to return just the function definition without a
// name. CouchDB does not like it when the functions have names.
//
// N.B. cannot define helper functions here, such as defining a function named
// bestDate that finds the most suitable date, as that function definition will
// not be included when defining the view. That would require using the !code
// macro in CouchDB, which probably requires storing JavaScript code in the
// vendor directory within the CouchDB installation.

/**
 * Convert the given view map function into a string with the "bestdate" code
 * inserted such that the search results have the most suitable date.
 *
 * @param {Function} func - map function.
 * @returns map function as a string.
 */
export function insertBestDate(func) {
  const s = func.toString()
  const t = s.replace('let bestdate', `let bestdate
    if (doc.userDate) {
      bestdate = doc.userDate
    } else if (doc.originalDate) {
      bestdate = doc.originalDate
    } else {
      bestdate = doc.importDate
    }`)
  return t
}

export const by_checksum = function (doc) {
  // special index, only need the document identifier
  if (doc.checksum) {
    emit(doc.checksum.toLowerCase())
  }
}

export const newborn = function (doc) {
  const notags = !Array.isArray(doc.tags) || doc.tags.length === 0
  const nocaption = doc.caption === undefined || doc.caption === null || doc.caption === ''
  const nolocation = doc.location === null || doc.location.label === undefined || doc.location.label === null || doc.location.label === ''
  if (notags && nocaption && nolocation) {
    let bestdate
    // newborn assets have no caption, tags, or location label (city and region
    // may be populated by reverse geocoding during import)
    //
    // use the import date for newborn as the query is in relation to when the
    // asset was imported
    emit(doc.importDate, [bestdate, doc.filename, doc.location, doc.mediaType])
  }
}

export const by_tag = function (doc) {
  if (doc.tags && Array.isArray(doc.tags)) {
    // see insertBestDate() for how bestdate works
    let bestdate
    doc.tags.forEach(function (tag) {
      emit(tag.toLowerCase(), [bestdate, doc.filename, doc.location, doc.mediaType])
    })
  }
}

export const by_date = function (doc) {
  // see insertBestDate() for how bestdate works
  let bestdate
  emit(bestdate, [bestdate, doc.filename, doc.location, doc.mediaType])
}

export const by_filename = function (doc) {
  // see insertBestDate() for how bestdate works
  let bestdate
  emit(doc.filename.toLowerCase(), [bestdate, doc.filename, doc.location, doc.mediaType])
}

export const by_location = function (doc) {
  if (doc.location) {
    // see insertBestDate() for how bestdate works
    let bestdate
    // emit only the location fields that have truthy values
    if (doc.location.label) {
      emit(doc.location.label.toLowerCase(), [bestdate, doc.filename, doc.location, doc.mediaType])
    }
    if (doc.location.city) {
      emit(doc.location.city.toLowerCase(), [bestdate, doc.filename, doc.location, doc.mediaType])
    }
    if (doc.location.region) {
      emit(doc.location.region.toLowerCase(), [bestdate, doc.filename, doc.location, doc.mediaType])
    }
  }
}

export const by_mimetype = function (doc) {
  // see insertBestDate() for how bestdate works
  let bestdate
  emit(doc.mediaType.toLowerCase(), [bestdate, doc.filename, doc.location, doc.mediaType])
}

export const all_location_records = function (doc) {
  if (doc.location) {
    const l = doc.location.label || ''
    const c = doc.location.city || ''
    const r = doc.location.region || ''
    // a completely empty location will be emitted as two tab characters
    emit(`${l}\t${c}\t${r}`, 1)
  }
}

export const all_location_parts = function (doc) {
  if (doc.location.label) {
    emit(doc.location.label.toLowerCase(), 1)
  }
  if (doc.location.city) {
    emit(doc.location.city.toLowerCase(), 1)
  }
  if (doc.location.region) {
    emit(doc.location.region.toLowerCase(), 1)
  }
}

export const all_tags = function (doc) {
  if (doc.tags && Array.isArray(doc.tags)) {
    doc.tags.forEach(function (tag) {
      emit(tag.toLowerCase(), 1)
    })
  }
}

export const all_years = function (doc) {
  if (doc.userDate) {
    emit(new Date(doc.userDate).getFullYear(), 1)
  } else if (doc.originalDate) {
    emit(new Date(doc.originalDate).getFullYear(), 1)
  } else {
    emit(new Date(doc.importDate).getFullYear(), 1)
  }
}

export const all_media_types = function (doc) {
  emit(doc.mediaType.toLowerCase(), 1)
}
