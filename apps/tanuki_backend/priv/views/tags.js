//
// View of all tags and their counts.
//
//!reduce: _count
function (doc) {
    if (doc.tags && Array.isArray(doc.tags)) {
        doc.tags.forEach(function (tag) {
            emit(tag.toLowerCase(), 1);
        });
    }
}
