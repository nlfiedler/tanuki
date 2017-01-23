//
// View from each location to the best date, filename, and checksum.
//
function (doc) {
    if (doc.location) {
        var date = null;
        if (doc.user_date) {
            date = doc.user_date;
        } else if (doc.exif_date) {
            date = doc.exif_date;
        } else if (doc.file_date) {
            date = doc.file_date;
        } else {
            date = doc.import_date;
        }
        emit(doc.location.toLowerCase(), [date, doc.file_name, doc.sha256]);
    }
}
