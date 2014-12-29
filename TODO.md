# TODO

## Overall

* Use [concrete](https://github.com/opscode/concrete) for dev-only dependencies
* Use [PropEr](http://proper.softlab.ntua.gr) for property-based testing
* Evaluate [QuickCheck](https://github.com/krestenkrab/triq) vs PropEr
* Look at [Elvis](https://github.com/inaka/elvis) for code style enforcement

## Web UI

### Action Items

1. Display thumbnails of the images in `tag.erl`
1. Fetch additional details of the image and display in `asset.erl`
1. Show all other tags in `tag.erl`
    * Clicking on one goes to `tags.erl` which shows assets with all selected tags
    * As with `tag.erl`, show all other tags; clicking on one adds to the list of tags
    * Show an `x` next to each tag; clicking removes tag from list and refreshes page
1. Organize the assets by date in `tag.erl`
1. Display available dates (year, then months)
1. Display assets by date (with pagination?)
1. Use ETS to cache the tags and such in memory of `tanuki_backend` process.
    * Cache invalidation via couchbeam change listeners.
1. Produce thumbnails for the videos and cache them
    * https://github.com/emedia-project/erlffmpeg
1. Would be good to Common Test the thumbnail generation code

### Implementation Details

* Starting tanuki backend on the server
    * Use Ubuntu Upstart
    * Use erl flags: -detached

## Incoming Processor

### Features

* Runs in the same process as the backend, kicked off by a timer (c.f. timer:send_after/2)
* Log actions during import
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
