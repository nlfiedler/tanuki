# TODO

## General Items

1. Replace all/most of the `.eex` pages with Elm components
    - Tags should show high ranking entries initially
        + Expander link/button to show all tags
        + Selected tags are always shown
    - Need to read `apiUrlPrefix` from configuration
    - Check how everything looks in Chrome and Firefox
1. Update emagick.rs to use latest magick-rust code; use that instead of invoking `convert`
    - Basically reversing commit `174fc11`
    - This includes removing the need to cache thumbnails on disk
    - Also removes the auto pruning of cached thumbnails
1. Add a "people" field
    - How to set this in import?
    - Need an admin screen to move a tag from "tags" to "people"
1. Change "locations" to "places" in the interface
1. Look for Rust bindings for FFmpeg libraries
1. Show video thumbnails using video HTML tag so they can play directly
1. Race condition in thinning of thumbnails may cause a request to fail
1. Instead of a "topic" field, perhaps an "occasion" field instead.
    - For instance, "christina birthday".
    - Add back to incoming processor using the "^" separator.
    - Would be on details/edit page for benefit of uploading.
    - Maybe don't bother exposing on main page, just another field like caption.
1. Request caching should be keyed by some unique value per browser session
    - With some upper limit on simultaneous cached queries
1. Option on `edit` page to rotate an image (some images lack orientation data)
    - How can Elm help with this feature?
1. Consider supporting browsing by year and month (likely without query caching or paging)
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
1. Update build instructions, or the build scripts
    - cd apps/tanuki_web/assets/elm && elm-package install -y
    - Does `npm install` need to be part of the build instructions in the `README`?
