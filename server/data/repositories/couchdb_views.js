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
// vendor directory in CouchDB itself.

/**
 * Convert the given view map function into a string with the "bestdate" code
 * inserted.
 *
 * @param {Function} func - map function.
 * @returns map function as a string.
 */
export function generateView(func) {
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

export const by_tag = function (doc) {
  if (doc.tags && Array.isArray(doc.tags)) {
    let bestdate
    doc.tags.forEach(function (tag) {
      emit(tag.toLowerCase(), [bestdate, doc.filename, doc.location, doc.mediaType])
    })
  }
}

export const by_date = function (doc) {
  let bestdate
  emit(bestdate, [bestdate, doc.filename, doc.location, doc.mediaType])
}

export const by_filename = function (doc) {
  let bestdate
  emit(doc.filename.toLowerCase(), [bestdate, doc.filename, doc.location, doc.mediaType])
}

export const by_location = function (doc) {
  if (doc.location) {
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
  let bestdate
  emit(doc.mediaType.toLowerCase(), [bestdate, doc.filename, doc.location, doc.mediaType])
}

export const all_locations = function (doc) {
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
