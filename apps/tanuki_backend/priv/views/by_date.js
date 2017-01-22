//
// View from the best available date to the checksum.
//
function (doc) {
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
    emit(date, [doc.file_name, doc.sha256]);
}
