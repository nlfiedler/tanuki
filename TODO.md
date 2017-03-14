# TODO

## General Items

1. Set up Elm
    - Use `apps/tanuki_web/lib/tanuki_web/web` in place of `web`
        + see https://gist.github.com/chrismccord/71ab10d433c98b714b75c886eff17357
    - https://medium.com/@diamondgfx/writing-a-full-site-in-phoenix-and-elm-a100804c9499
    - https://medium.com/@diamondgfx/writing-a-full-site-in-phoenix-and-elm-a4622c5130f5
    - http://www.cultivatehq.com/posts/phoenix-elm-2/
    - Edit the `elm-package.json` to hold relevant information

```
$ cd apps/tanuki_web/lib/tanuki_web
$ mkdir web/elm
$ cd web/elm
$ elm-package install
```
1. Build tag list in Elm, fetching list via `/api/tags`
    - Need an `/api/tags` handler
    - Returned JSON includes tag names and asset counts
    - Elm displays (in alpha order) those tags whose counts exceed a threshold
    - Add a `>>` button that then shows the rest of the tags (in alpha order)
1. Consider using Elm to replace the guts of the `.eex` pages
1. Race condition in thinning of thumbnails may cause a request to fail
1. Instead of a "topic" field, perhaps an "occasion" field instead.
    - For instance, "christina birthday".
    - Add back to incoming processor using the "^" separator.
    - Would be on details/edit page for benefit of uploading.
    - Maybe don't bother exposing on main page, just another field like caption.
1. Request caching should be keyed by some unique value per browser session
    - With some upper limit on simultaneous cached queries
1. Option on `edit` page to rotate an image (some images lack orientation data)
1. Write tests for the `tanuki_web` application
1. Consider supporting browsing by year and month (likely without query caching or paging)
1. Fix image references in error view
    - When Phoenix has an error, it tries to refer to default images
1. Add functions to admin page:
    - Button to perform database compaction
1. Completion for tags, location
1. Bulk edit feature
    - Design a query page that allows searching on several fields (tags, date, location)
    - Use a temporary view (http://docs.couchdb.org/en/1.6.1/api/database/temp-views.html)
    - Multi-select the displayed results
    - Present a form for changing one or more fields of the selected assets
1. Client-side validation of the custom date field on edit page.
