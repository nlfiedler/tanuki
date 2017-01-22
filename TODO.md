# TODO

## General Items

1. Add an "/admin" scope for various functions (use "pencil" or "dashboard" glyph)
    - Renaming a tag across all documents
    - Renaming a location across all documents
    - Changing a tag to a location across all documents
    - Button to kick off the incoming processor
    - Button to perform database compaction
1. Show list of known locations
1. For large numbers of pages, maybe some "skip" links
    - On the `year` page, show links to jump to the page with the first asset in each month
    - On the `index` page, show links to jump by 10 or 100 pages at a time.
1. Option on `edit` page to rotate an image (some images lack orientation data).
1. Ensure QuickTime videos play in the browser
1. To help reduce tag clutter, hide the tags whose count falls below a threshold
    - Show the full list of tags dynamically using JavaScript (via a link)
1. Produce thumbnails for the videos and cache them
    - https://github.com/emedia-project/erlffmpeg
1. Can we produce thumbnails for AVI files?
1. Write tests for the `tanuki_web` application
1. Consider supporting browsing by year and month (likely without query caching or paging).
1. Fix image references in error view
    - When Phoenix has an error, it tries to refer to default images
