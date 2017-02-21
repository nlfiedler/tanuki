# TODO

## General Items

1. Pagination links should have "..." where additional pages exist.
1. Race condition in thinning of thumbnails may cause a request to fail
1. Instead of a "topic" field, perhaps an "occasion" field instead.
    - For instance, "christina birthday".
    - Add back to incoming processor using the "^" separator.
    - Would be on details/edit page for benefit of uploading.
    - Maybe don't bother exposing on main page, just another field like caption.
1. Request caching should be keyed by some unique value per browser session
    - With some upper limit on simultaneous cached queries
1. For large numbers of pages, maybe some "skip" links
    - On the `year` page, show links to jump to the page with the first asset in each month
    - On the `index` page, show links to jump to 10% mark, 20% mark, and so on
    - On the `location` page, show links to jump to 10% mark, 20% mark, and so on
1. Option on `edit` page to rotate an image (some images lack orientation data)
1. Ensure QuickTime videos play in the browser
1. To help reduce tag clutter, hide the tags whose count falls below a threshold
    - Show the full list of tags dynamically using JavaScript (via a link)
1. Use `FFprobe.duration("/path/to/input.avi")` to get duration of vides during import
    - Add 1-off script to add the "duration" field to all existing video documents
    - Show duration value on asset detail page
1. Write tests for the `tanuki_web` application
1. Consider supporting browsing by year and month (likely without query caching or paging)
1. Fix image references in error view
    - When Phoenix has an error, it tries to refer to default images
1. Add functions to admin page:
    - Button to kick off the incoming processor
    - Button to perform database compaction
