# TODO

## Action Items

1. Set up rc-style launcher script (like couchdb) for starting/stopping in FreeBSD
    * https://www.freebsd.org/doc/en_US.ISO8859-1/articles/rc-scripting/article.html
1. Add pagination support to `tag.erl`
    * Look at the paginate plugin: https://github.com/choptastic/paginate
    * http://guide.couchdb.org/draft/recipes.html
        * Request rows_per_page + 1 rows from the view
        * Display rows_per_page rows
        * Store the +1 row as next_startkey and next_startkey_docid
        * As page information, keep startkey and next_startkey
        * Use the next_* values to create the next link
        * Use the others (startkey?) to create the previous link
    * An entirely different approach: https://gist.github.com/amedeo/820412
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
    * Could use https://github.com/nitrogen/simple_cache
1. Produce thumbnails for the videos and cache them
    * https://github.com/emedia-project/erlffmpeg
1. Show list of known locations along with the list of tags
1. Show list of known topics along with the list of tags, locations
