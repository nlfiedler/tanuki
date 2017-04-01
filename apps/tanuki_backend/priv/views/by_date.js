//
// View from the best available date to the filename, checksum, and location.
//
function (doc) {
    var date = null;
    if (doc.user_date) {
        date = doc.user_date;
    } else if (doc.original_date) {
        date = doc.original_date;
    } else if (doc.file_date) {
        date = doc.file_date;
    } else {
        date = doc.import_date;
    }
    var location = null;
    if (doc.location) {
        location = doc.location.toLowerCase();
    } else {
        location = "";
    }
    emit(date, [doc.file_name, doc.sha256, location]);
}
