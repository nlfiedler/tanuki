//
// View of checksums to mimetypes.
//
// no reduce option
function(doc) {
    if (doc.sha256) {
        emit(doc.sha256, doc.mimetype);
    }
}
