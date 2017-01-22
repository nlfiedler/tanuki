//
// View of all years and their counts.
//
//!reduce: _count
function (doc) {
    if (doc.user_date) {
        emit(doc.user_date[0], 1);
    } else if (doc.exif_date) {
        emit(doc.exif_date[0], 1);
    } else if (doc.file_date) {
        emit(doc.file_date[0], 1);
    } else {
        emit(doc.import_date[0], 1);
    }
}
