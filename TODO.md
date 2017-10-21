# TODO

## General Items

1. Browse photos by groups taken around the same date
1. Need to read `apiUrlPrefix` in Elm code from configuration file
1. Support HEIC/HEIF file formats
    - Need ImageMagick support, see issue #507 on GitHub
1. Add a "people" field
    - How to set this in import?
    - Need an admin screen to move a tag from "tags" to "people"
    - Maybe a generic "move" action:
        + Given a list of tags to be moved...
        + And the name of a field (e.g. "people")...
        + Move the given tags to the named field
1. Change "locations" to "places" in the interface
1. Look for Rust bindings for FFmpeg libraries
1. Show video thumbnails using video HTML tag so they can play directly
    - Otherwise you just get the broken image placeholder
1. Instead of a "topic" field, perhaps an "occasion" field instead.
    - For instance, "christina birthday".
    - Add back to incoming processor using the "^" separator.
    - Would be on details/edit page for benefit of uploading.
    - Maybe don't bother exposing on main page, just another field like caption.
1. Find out how to handle mnesia schema changes over time
    - e.g. adding a new field to the thumbnails cache, need to wipe existing table(s)
1. Option on `edit` page to rotate an image (some images lack orientation data)
    - How can Elm help with this feature?
1. Make the number of cached thumbnails depend on the available memory
1. Consider supporting browsing by year and month (likely without paging)
1. Fix image references in error view
    - When Phoenix has an error, it tries to refer to default images
1. Add functions to admin page:
    - Button to perform database compaction
1. Completion for tags, people, places
1. Bulk edit feature
    - Design a query page that allows searching on several fields (tags, date, location)
    - Use a temporary view (http://docs.couchdb.org/en/1.6.1/api/database/temp-views.html)
    - Multi-select the displayed results
    - Present a form for changing one or more fields of the selected assets
