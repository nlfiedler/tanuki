//
// View of all locations and their counts.
//
//!reduce: _count
function (doc) {
    if (doc.location) {
        emit(doc.location.toLowerCase(), 1);
    }
}
