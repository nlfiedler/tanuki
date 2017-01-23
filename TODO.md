# TODO

## General Items

1. Show list of known locations
1. For large numbers of pages, maybe some "skip" links
    - On the `year` page, show links to jump to the page with the first asset in each month
    - On the `index` page, show links to jump to 10% mark, 20% mark, and so on
1. Option on `edit` page to rotate an image (some images lack orientation data)
1. Ensure QuickTime videos play in the browser
1. To help reduce tag clutter, hide the tags whose count falls below a threshold
    - Show the full list of tags dynamically using JavaScript (via a link)
1. Produce thumbnails for the videos and cache them
    - https://github.com/emedia-project/erlffmpeg
1. Can we produce thumbnails for AVI files?
1. Write tests for the `tanuki_web` application
1. Consider supporting browsing by year and month (likely without query caching or paging)
1. Fix image references in error view
    - When Phoenix has an error, it tries to refer to default images
1. Add functions to admin page:
    - Button to kick off the incoming processor
    - Button to perform database compaction
