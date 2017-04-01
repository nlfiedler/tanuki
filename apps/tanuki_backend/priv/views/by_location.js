//
// View from each location to the best date, filename, and checksum.
//
function (doc) {
    if (doc.location) {
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
        var location = doc.location.toLowerCase();
        // keep the included values the same across by_date, by_location, by_tag
        emit(location, [date, doc.file_name, doc.sha256, location]);
    }
}
