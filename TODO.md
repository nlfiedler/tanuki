# TODO

## General Items

1. Change `install_designs/1` to read `*.js` files instead of `*.json`
    - Any `//!reduce:...` comment indicates if and how reduce occurs
    - The non-comment lines are processed thusly:
        + Remove leading whitespace
        + Replace newlines with spaces
        + Sort the entries by name of file (minus extension), hereafter called `filename`
        + Store resulting text into `views`: `#{filename}`: `map`
        + The "reduce" value, if any, goes in `views`: `#{filename}`: `reduce`
    - Generate the rest of the fields thusly:
        + `_id`: `_design/assets`
        + `language`: `javascript`
1. Are the many `to_string/1` calls in `page_controller` really necessary?
1. Add an "/admin" scope for various functions
    - Renaming a tag across all documents
    - Renaming a location across all documents
    - Changing a tag to a location across all documents
    - Button to kick off the incoming processor
1. Get the overlays working in the release (see `rel/config.exs`)
1. See if there is an easy way to get the `Version` file working again
    - Maybe just generate it using some code in `rel/config.exs`
1. Read about deployment in Phoenix:
    - http://www.phoenixframework.org/docs/advanced-deployment
    - https://medium.com/@brucepomeroy/create-an-elixir-umbrella-project-containing-a-phoenix-app-and-build-a-release-with-distillery-46371f2617df#.oinfggn3b
1. Get the `COOKIE` envar thing working for `:prod` in `rel/config.exs`
1. Look for bootstrap css guide to select a good style
1. Fix image references in error view
1. Find out where the Elixir apps will be writing their logs
1. Ensure QuickTime videos play in the browser
1. Produce thumbnails for the videos and cache them
    - https://github.com/emedia-project/erlffmpeg
1. Can we produce thumbnails for AVI files?
1. Show list of known locations
1. Write tests for the `tanuki_web` application
1. To help reduce tag clutter, hide the tags whose count falls below a threshold
    - Show the full list of tags dynamically using JavaScript (via a link)
1. Consider supporting browsing by year and month (likely without query caching or paging).

## Documentation

1. Write an architecture document that explains the components:
    * CouchDB
        - couchbeam
    * Incoming
    * Backend
    * Frontend
        - Phoenix
            + Cowboy
            + Plug
        - Elixir
    * Describe how pages are rendered (Phoenix endpoints, router, controllers, views)
    * Describe how the incoming processor works
        - Configuring incoming directory
        - Naming of folder
        - Where assets end up
