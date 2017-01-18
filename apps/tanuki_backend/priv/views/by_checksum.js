//
// View from checksum to mimetype.
//
function (doc) {
    if (doc.sha256) {
        emit(doc.sha256, doc.mimetype);
    }
}
