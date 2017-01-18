//

// tags view with a reduce by count

// and bunches of blank lines

//!reduce: _count

function (doc) {

    if (doc.tags && Array.isArray(doc.tags)) {

        doc.tags.forEach(function (tag) {

            emit(tag.toLowerCase(), 1);

        });

    }

}
