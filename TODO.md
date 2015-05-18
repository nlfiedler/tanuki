# TODO

## Overall

* Use [concrete](https://github.com/opscode/concrete) for dev-only dependencies
* Use [PropEr](http://proper.softlab.ntua.gr) for property-based testing
* Evaluate [QuickCheck](https://github.com/krestenkrab/triq) vs PropEr
* Look at [Elvis](https://github.com/inaka/elvis) for code style enforcement

## Web UI

### Action Items

1. Set up rebar to build merge_records.escript the way the steps were done in Make-a-Lisp
    * Generate escript from .erl file
    * Include dependencies so file is self-contained
1. Convert the Vagrant files to Docker files
1. Add pagination support to `tag.erl`
    * http://guide.couchdb.org/draft/recipes.html
        * Request rows_per_page + 1 rows from the view
        * Display rows_per_page rows
        * Store the +1 row as next_startkey and next_startkey_docid
        * As page information, keep startkey and next_startkey
        * Use the next_* values to create the next link
        * Use the others (startkey?) to create the previous link
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

### Implementation Details

* Starting tanuki backend on the server
    * Use Ubuntu Upstart
    * Use erl flags: -detached

## Incoming Processor

### Features

* Send a daily email report of everything that was imported
    * Include the names of files and their checksums
    * Organize by tags

### Implementation Details

* Be sure to write thorough unit tests to guard against accidental data loss. The data loader is the most fragile in the system because it adds records to the database and moves files in the file system. These need to be performed as an atomic transaction.
    * Attempt to move the asset into place first; if that fails stop immediately.
    * If the attempt to insert the document into the database fails, revert the asset move.

### Installation and Configuration

* Having a `tanuki` user is a good idea for file ownership and permissions.
* The incoming processor and web stack should run as the tanuki user.
