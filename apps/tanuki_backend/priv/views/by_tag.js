//
// View from each tag to the best date, filename, checksum, and location.
//
function (doc) {
    if (doc.tags && Array.isArray(doc.tags)) {
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
        doc.tags.forEach(function (tag) {
            emit(tag.toLowerCase(), [date, doc.file_name, doc.sha256, location]);
        });
    }
}
