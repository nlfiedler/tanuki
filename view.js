
/* global emit */

const by_tag = function (doc) {
  if (doc.tags && Array.isArray(doc.tags)) {
    let bestdate
    doc.tags.forEach(function (tag) {
      emit(tag.toLowerCase(), [bestdate, doc.filename, doc.location, doc.mediaType])
    })
  }
}

const s = by_tag.toString()
const t = s.replace('let bestdate', `let bestdate
    if (doc.userDate) {
      bestdate = doc.userDate
    } else if (doc.originalDate) {
      bestdate = doc.originalDate
    } else {
      bestdate = doc.importDate
    }`)

console.log(t)
