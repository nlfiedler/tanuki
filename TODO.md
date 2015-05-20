# TODO

## Action Items

1. Use `gen_smtp` in `tanuki_incoming` app to send email report for each import
    * Include the names of files and their checksums
    * Organize by tags
1. If tanuki_incoming fails to insert new CouchDB document, revert the asset move
1. Add pagination support to `tag.erl`
    * http://guide.couchdb.org/draft/recipes.html
        * Request rows_per_page + 1 rows from the view
        * Display rows_per_page rows
        * Store the +1 row as next_startkey and next_startkey_docid
        * As page information, keep startkey and next_startkey
        * Use the next_* values to create the next link
        * Use the others (startkey?) to create the previous link
1. Store tag selection in page state (e.g. `wf:state(Key, Value)`)
1. Show an `x` next to each tag on `tag.erl` to drop that tag from the query
1. Get the list of tags in `tag.erl` to be along the side of the images
1. Have the thumbnails on `tag.erl` appear in a grid format
1. Organize the assets by date in `tag.erl`
1. Have `asset.erl` show a larger version of the image instead of thumbnail
1. Fetch additional details of the image for display in `asset.erl`
1. Display available dates (year, then months) on `index.erl`
1. Display assets by date (with pagination?) in a new `date.erl` page
1. Use ETS to cache the tags and such in memory of `tanuki_backend` process.
    * Cache invalidation via couchbeam change listeners.
1. Produce thumbnails for the videos and cache them
    * https://github.com/emedia-project/erlffmpeg
1. Would be good to Common Test the thumbnail generation code
1. Show list of known locations along with the list of tags
1. Will need pagination for `tag.erl`
1. Look into using https://github.com/ShoreTel-Inc/erld with Upstart to manage process
1. Find the log for tanuki and add to logwatch
1. Look at https://github.com/evanmiller/erl_img for possible image scaling library
    * Would replace ImageMagick and emagick
