module View exposing (..)

import Dict
import Forms
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (on, onClick, onInput, onSubmit, targetValue)
import Html.Keyed
import Json.Decode
import Json.Encode exposing (string)
import List.Extra exposing (greedyGroupsOf)
import Messages exposing (..)
import Mimetypes
import Model exposing (..)
import RemoteData
import Routing exposing (Route(..))


view : Model -> Html Msg
view model =
    case model.route of
        HomeIndexRoute ->
            indexPage model

        UploadRoute ->
            uploadPage model

        ShowAssetRoute id ->
            viewAsset model

        EditAssetRoute id ->
            editAsset model

        NotFoundRoute ->
            notFoundView


indexPage : Model -> Html Msg
indexPage model =
    div [ ]
        [ tagSelector model
        , yearSelector model
        , locationSelector model
        , viewThumbnails model
        ]


uploadPage : Model -> Html Msg
uploadPage model =
    let
        helpDisplay =
            if model.hasDragSupport then
                "block"
            else
                "none"
        ( uploadDisabled, uploadFilename ) =
            case model.uploadFilename of
                Just fname ->
                    ( False, fname )
                Nothing ->
                    ( True, "" )
    in
        div [ ]
            [ Html.form
                [ action "/import"
                , method "post"
                , enctype "multipart/form-data"
                ]
                [ div [ class "control" ]
                    [ div [ class "file has-name is-boxed" ]
                        [ label [ class "file-label" ]
                            [ input
                                [ class "file-input"
                                , type_ "file"
                                , multiple False
                                , name "asset"
                                , required True
                                , on "change" (Json.Decode.map UploadSelection targetValue)
                                ] [ ]
                            , span [ class "file-cta" ]
                                [ span [ class "file-icon" ]
                                    [ i [ class "fas fa-upload" ] [ ] ]
                                , span [ class "file-label" ]
                                    [ text "Choose a file…" ]
                                ]
                            , span [ class "file-name" ] [ text uploadFilename ]
                            ]
                        ]
                    ]
                , p [ style [ ("display", helpDisplay) ]
                    , class "help"
                    ] [ text "You can drag and drop a file on the above control" ]
                , div [ class "control" ]
                    [ input
                        [ class "button is-primary"
                        , type_ "submit"
                        , value "Upload"
                        , disabled uploadDisabled
                        ]
                        [ ]

                    ]
                ]
            ]


tagSelector : Model -> Html Msg
tagSelector model =
    let
        header =
            span [ class "tag is-info" ] [ text "Tags" ]
        footer =
            allTagsToggle model
        entries =
            (header :: viewTagList ToggleTag model) ++ [footer]
    in
        div [ class "tags" ] entries


allTagsToggle : Model -> Html Msg
allTagsToggle model =
    if model.showingAllTags then
        a
            [ class "tag is-light"
            , href "#"
            , title "Hide some tags"
            , onClick ToggleAllTags ] [ text "<<" ]
    else
        let
            tagList =
                RemoteData.withDefault [] model.tagList
            result =
                if (List.length tagList <= 25) then
                    text ""
                else
                    a
                        [ class "tag is-light"
                        , href "#"
                        , title "Show all tags"
                        , onClick ToggleAllTags ] [ text ">>" ]
        in
            result

viewTagList : (String -> msg) -> Model -> List (Html msg)
viewTagList msg model =
    case model.tagList of
        RemoteData.NotAsked ->
            [ text "Initializing..." ]

        RemoteData.Loading ->
            [ text "Loading tags..." ]

        RemoteData.Failure error ->
            [ text (toString error) ]

        RemoteData.Success list ->
            let
                -- filter tags by their count, if hiding, else include all
                tags =
                    if model.showingAllTags || (List.length list <= 25) then
                        list
                    else
                        selectTopTags list
            in
                List.map (viewTagItem msg) tags


{- Return tags that match several criteria.

Returns the top 25 tags sorted by the number of assets they select, and then
sorted by the label. Tags that are currently selected by the user are always
included in the result.

A clever alternative would be to find the elbow/knee of the data but that
requires a lot more math than Elm is really suitable for.

-}
selectTopTags : TagList -> TagList
selectTopTags tags =
    let
        -- Get the selected tags into a dict.
        selectedTags =
            List.filter (.selected) tags
        mapInserter v d =
            Dict.insert v.label v d
        selectedTagsDict =
            List.foldl mapInserter Dict.empty selectedTags
        -- Get the top N tags by count.
        tagSorter a b =
            case compare a.count b.count of
                LT -> GT
                EQ -> EQ
                GT -> LT
        sortedTopTags =
            List.take 25 (List.sortWith tagSorter tags)
        -- Merge those two sets into one.
        mergedTagsDict =
            List.foldl mapInserter selectedTagsDict sortedTopTags
    in
        -- Extract the values and sort by label.
        List.sortBy .label (Dict.values mergedTagsDict)


viewTagItem : (String -> msg) -> Tag -> Html msg
viewTagItem msg entry =
    let
        tagClass =
            if entry.selected then
                "tag is-dark"
            else
                "tag is-light"
    in
        a
            [ class tagClass
            , href "#"
            , title (toString entry.count)
            , onClick (msg entry.label)
            ] [ text entry.label ]


yearSelector : Model -> Html Msg
yearSelector model =
    let
        header =
            span [ class "tag is-info" ] [ text "Years" ]
        entries =
            header :: viewYearList ToggleYear model.yearList
    in
        div [ class "tags" ] entries


viewYearList : (Int -> msg) -> GraphData YearList -> List (Html msg)
viewYearList msg entries =
    case entries of
        RemoteData.NotAsked ->
            [ text "Initializing..." ]

        RemoteData.Loading ->
            [ text "Loading years..." ]

        RemoteData.Failure error ->
            [ text (toString error) ]

        RemoteData.Success list ->
            List.map (viewYearItem msg) list


viewYearItem : (Int -> msg) -> Year -> Html msg
viewYearItem msg entry =
    let
        tagClass =
            if entry.selected then
                "tag is-dark"
            else
                "tag is-light"
    in
        a
            [ class tagClass
            , href "#"
            , title (toString entry.count)
            , onClick (msg entry.year)
            ] [ text (toString entry.year) ]


locationSelector : Model -> Html Msg
locationSelector model =
    let
        header =
            span [ class "tag is-info" ] [ text "Locations" ]
        footer =
            allLocationsToggle model
        entries =
            (header :: viewLocationList ToggleLocation model) ++ [footer]
    in
        div [ class "tags" ] entries


allLocationsToggle : Model -> Html Msg
allLocationsToggle model =
    if model.showingAllLocations then
        a
            [ class "tag is-light"
            , href "#"
            , title "Hide some locations"
            , onClick ToggleAllLocations
            ] [ text "<<" ]
    else
        let
            locationList =
                RemoteData.withDefault [] model.locationList
            result =
                if (List.length locationList <= 25) then
                    text ""
                else
                    a
                        [ class "tag is-light"
                        , href "#"
                        , title "Show all locations"
                        , onClick ToggleAllLocations
                        ] [ text ">>" ]
        in
            result


viewLocationList : (String -> msg) -> Model -> List (Html msg)
viewLocationList msg model =
    case model.locationList of
        RemoteData.NotAsked ->
            [ text "Initializing..." ]

        RemoteData.Loading ->
            [ text "Loading locations..." ]

        RemoteData.Failure error ->
            [ text (toString error) ]

        RemoteData.Success list ->
            let
                -- filter location by their count, if hiding, else include all
                locations =
                    if model.showingAllLocations || (List.length list <= 25) then
                        list
                    else
                        selectTopLocations list
            in
                List.map (viewLocationItem msg) locations


{- Return locations that match several criteria.

Returns the top 25 locations sorted by the number of assets they select, and
then sorted by the label. Locations that are currently selected by the user are
always included in the result.

A clever alternative would be to find the elbow/knee of the data but that
requires a lot more math than Elm is really suitable for.

-}
selectTopLocations : LocationList -> LocationList
selectTopLocations locations =
    let
        -- Get the selected locations into a dict.
        selectedLocations =
            List.filter (.selected) locations
        mapInserter v d =
            Dict.insert v.label v d
        selectedLocationsDict =
            List.foldl mapInserter Dict.empty selectedLocations
        -- Get the top N locations by count.
        locationSorter a b =
            case compare a.count b.count of
                LT -> GT
                EQ -> EQ
                GT -> LT
        sortedTopLocations =
            List.take 25 (List.sortWith locationSorter locations)
        -- Merge those two sets into one.
        mergedLocationsDict =
            List.foldl mapInserter selectedLocationsDict sortedTopLocations
    in
        -- Extract the values and sort by label.
        List.sortBy .label (Dict.values mergedLocationsDict)


viewLocationItem : (String -> msg) -> Location -> Html msg
viewLocationItem msg entry =
    let
        tagClass =
            if entry.selected then
                "tag is-dark"
            else
                "tag is-light"
    in
        a
            [ class tagClass
            , href "#"
            , title (toString entry.count)
            , onClick (msg entry.label)
            ] [ text entry.label ]


viewThumbnails : Model -> Html Msg
viewThumbnails model =
    case model.assetList of
        RemoteData.NotAsked ->
            div [ class "notification" ]
                [ text "Make a selection above to display assets." ]

        RemoteData.Loading ->
            text "Loading thumbnails..."

        RemoteData.Failure error ->
            text (toString error)

        RemoteData.Success list ->
            let
                -- Use the awesome, flexible bulma columns and then force them
                -- to be thirds, so we avoid the images shrinking needlessly.
                -- Then wrap the individual column elements in a "columns",
                -- one for each row, and that is collected in a container.
                -- Basically a primitive table.
                cells =
                    List.map viewThumbnailItem list.entries
                groups =
                    greedyGroupsOf 3 cells
                rows =
                    List.map (div [ class "columns" ]) groups
                paging =
                    -- skip the pagination links if there is nothing to page
                    if list.total_entries > pageSize && List.length list.entries > 0 then
                        paginationList model.pageNumber list
                    else
                        [ text "" ]
            in
                div [ class "container" ] (rows ++ paging)


viewThumbnailItem : AssetSummary -> Html Msg
viewThumbnailItem entry =
    let
        -- any asset that fails to produce a thumbnail will get a placeholder
        imgSrc =
            if entry.thumbless then
                "/images/" ++ brokenThumbnailPlaceholder entry.file_name
            else
                "/thumbnail/" ++ entry.id
        baseImgAttrs =
            [ src imgSrc
            , alt entry.file_name
            , style [ ("width", "auto") ]
            ]
        imgAttrs =
            if entry.thumbless then
                baseImgAttrs
            else
                baseImgAttrs ++ [ on "error" (Json.Decode.succeed (ThumblessAsset entry.id)) ]
    in
        div [ class "column is-one-third" ]
            [ div [ class "card" ]
                [ div [ class "card-content"
                      , onClick <| NavigateTo <| ShowAssetRoute entry.id
                      ]
                    -- overflow only works on block elements, so apply it here;
                    -- long file names with "break" characters (e.g. '-') will
                    -- wrap automatically anyway
                    [ figure [ class "image", style [("overflow", "hidden")] ]
                        [ img imgAttrs [ ]
                        , small [ ] [ text (intToDateString entry.date) ]
                        , br [ ] [ ]
                        , small [ ] [ text entry.file_name ]
                        ]
                    ]
                ]
            ]


-- Map the asset filename to the filename of an image known to be available
-- on the backend (in public/images).
brokenThumbnailPlaceholder : String -> String
brokenThumbnailPlaceholder filename =
    case Mimetypes.filenameToMimetype filename of
        Mimetypes.Image ->
            "file-picture.png"
        Mimetypes.Video ->
            "file-video-2.png"
        Mimetypes.Pdf ->
            "file-acrobat.png"
        Mimetypes.Audio ->
            "file-music-3.png"
        Mimetypes.Text ->
            "file-new-2.png"
        Mimetypes.Unknown ->
            "file-new-1.png"


paginationList : Int -> AssetList -> List (Html Msg)
paginationList currentPage list =
    let
        -- i am not proud of how huge and ugly this is...
        totalPages =
            ceiling ((toFloat list.total_entries) / (toFloat pageSize))
        desiredLower =
            currentPage - 5
        desiredUpper =
            currentPage + 4
        ( lower, upper ) =
            if desiredLower <= 1 then
                ( 2, Basics.min (desiredUpper + (abs desiredLower)) (totalPages - 1) )
            else if desiredUpper >= totalPages then
                ( Basics.max (desiredLower - (desiredUpper - totalPages)) 2, (totalPages - 1) )
            else
                ( desiredLower, desiredUpper )
        numberedLinks =
            List.map (paginationLink currentPage) (List.range lower upper)
        firstLink =
            paginationLink currentPage 1
        prevLink =
            if (currentPage - 10) <= 1 then
                Nothing
            else
                namedPaginationLink "«" (currentPage - 10)
        ellipsis =
            span [ property "innerHTML" <| string "&hellip;" ] [ ]
        preDots =
            if lower > 2 then
                Just ( ",...", li [ class "pagination-ellipsis" ] [ span [ ] [ ellipsis ] ] )
            else
                Nothing
        postDots =
            if upper < (totalPages - 1) then
                Just ( "...,", li [ class "pagination-ellipsis" ] [ span [ ] [ ellipsis ] ] )
            else
                Nothing
        nextLink =
            if (currentPage + 10) >= totalPages then
                Nothing
            else
                namedPaginationLink "»" (currentPage + 10)
        lastLink =
            paginationLink currentPage totalPages
        maybeLinks =
            [firstLink, prevLink, preDots] ++ numberedLinks ++ [postDots, nextLink, lastLink]
        linkList =
            Html.Keyed.ul [ class "pagination-list" ] (justLinksExtractor maybeLinks)
    in
        [ nav
            [ class "pagination is-centered"
            , attribute "role" "navigation"
            ] [ linkList ] ]


namedPaginationLink : String -> Int -> Maybe ( String, Html Msg )
namedPaginationLink label page =
    Just ( toString page, li [ ] [ a [ onClick <| Paginate page ] [ text label ] ] )


paginationLink : Int -> Int -> Maybe ( String, Html Msg )
paginationLink currentPage page =
    let
        classes =
            classList
                [ ("pagination-link", True)
                , ( "is-current", currentPage == page )
                ]
    in
        Just ( toString page
            , li [ ]
                [ a [ classes, onClick <| Paginate page ] [ text (toString page) ] ]
            )


{- Extract the Maybe links, dropping the Nothings.
-}
justLinksExtractor : List ( Maybe ( String, Html Msg ) ) -> List ( ( String, Html Msg ) )
justLinksExtractor maybeLinks =
    let
        maybe_filter e =
            case e of
                Just l ->
                    True
                Nothing ->
                    False
        just_extractor e =
            Maybe.withDefault ( "a", text "a" ) e
    in
        List.map just_extractor (List.filter maybe_filter maybeLinks)


viewAsset : Model -> Html Msg
viewAsset model =
    case model.asset of
        RemoteData.NotAsked ->
            text "Details coming soon..."

        RemoteData.Loading ->
            text "Loading asset..."

        RemoteData.Failure error ->
            warningMessage (toString error) backToHomeLink

        RemoteData.Success asset ->
            div [ class "container" ]
                [ (viewAssetPreviewPanel asset) ]


viewAssetPreviewPanel : AssetDetails -> Html Msg
viewAssetPreviewPanel asset =
    div [ class "card" ]
        [ div [ class "card-header" ]
            [ p [ class "card-header-title" ] [ text asset.file_name ]
            , a [ class "card-header-icon"
                , onClick <| NavigateTo <| EditAssetRoute asset.id
                ]
                [ span [ class "icon" ] [ i [ class "fa fa-edit" ] [ ] ] ]
            ]
        , div [ class "card-image has-text-centered" ] [ viewAssetPreview asset ]
        , div [ class "card-content" ] [ viewAssetDetails asset ]
        ]


viewAssetPreview : AssetDetails -> Html Msg
viewAssetPreview asset =
    if String.startsWith "video/" asset.mimetype then
        video [ style [ ("width", "100%"), ("height", "100%") ]
              , controls True, preload "auto" ]
            [ source [ src ("/asset/" ++ asset.id)
                     , type_ (assetMimeType asset.mimetype) ] [ ]
            , text "Bummer, your browser does not support the HTML5"
            , code [ ] [ text "video" ]
            , text "tag."
            ]
    else
        a [ href ("/asset/" ++ asset.id) ]
            [ figure [ class "image" ]
                -- styles for getting the image to center and not resize
                [ img [ style [("display", "inline"), ("width", "auto")]
                      , src ("/preview/" ++ asset.id)
                      , alt asset.file_name ] [ ]
                ]
            ]


viewAssetDetails : AssetDetails -> Html Msg
viewAssetDetails asset =
    let
        part1 =
            [ ( "Date", (intToDateString asset.datetime) )
            , ( "Size", (toString asset.file_size) )
            , ( "SHA256", asset.id )
            ]
        -- The duration will be placed in the middle since it seems to fit
        -- better there than after the tags, even though that would have
        -- been easier to write.
        part2 =
            [ ( "Location", Maybe.withDefault "" asset.location )
            , ( "Caption", Maybe.withDefault "" asset.caption )
            , ( "Tags", String.join ", " asset.tags )
            ]
        rows =
            case asset.duration of
                Just secs ->
                    part1 ++ [ ( "Duration", (toString secs) ++ " seconds" ) ] ++ part2

                Nothing ->
                    part1 ++ part2
        table_maker ( label, data ) =
            [ tr [ ]
                [ th [ ] [ text label ]
                , td [ ] [ text data ]
                ]
            ]
    in
        table [ class "table is-striped is-fullwidth" ]
            [ tbody [ ] (List.concatMap (table_maker) rows) ]



notFoundView : Html Msg
notFoundView =
    warningMessage "Page not found" backToHomeLink


warningMessage : String -> Html Msg -> Html Msg
warningMessage message content =
    div [ class "container" ]
        [ article [ class "message is-warning" ]
            [ div [ class "message-header" ] [ text "Warning" ]
            , div [ class "message-body" ]
                [ div [ class "content", style [("font-family", "monospace")] ]
                    [ text message ] ]
            ]
        , content
        ]


backToHomeLink : Html Msg
backToHomeLink =
    a [ onClick <| NavigateTo HomeIndexRoute ]
        [ text "← Back to home" ]


{- Pretend that quicktime videos are really MP4.

Which is technically true most of the time, and it gets Google Chrome to
show the video properly without having to install any plugins.

-}
assetMimeType : String -> String
assetMimeType mimetype =
    if mimetype == "video/quicktime" then
        "video/mp4"
    else
        mimetype


editAsset : Model -> Html Msg
editAsset model =
    case model.asset of
        RemoteData.NotAsked ->
            text "Edit form preparing..."

        RemoteData.Loading ->
            text "Loading asset..."

        RemoteData.Failure error ->
            warningMessage (toString error) backToHomeLink

        RemoteData.Success asset ->
            div [ class "container" ]
                [ (editAssetPanel model.assetEditForm asset) ]


editAssetPanel : Forms.Form -> AssetDetails -> Html Msg
editAssetPanel form asset =
    div [ class "card" ]
        [ div [ class "card-header" ]
            [ p [ class "card-header-title" ]
                [ text (intToDateString asset.datetime)
                , text ", "
                , text asset.file_name
                ]
            ]
        , div [ class "card-image has-text-centered" ] [ viewAssetPreview asset ]
        , div [ class "card-content" ] [ (editAssetForm form asset) ]
        ]


editAssetForm : Forms.Form -> AssetDetails -> Html Msg
editAssetForm form asset =
    let
        location =
            Forms.formValueWithDefault (Maybe.withDefault "" asset.location) form "location"
        caption =
            Forms.formValueWithDefault (Maybe.withDefault "" asset.caption) form "caption"
        userDate =
            Forms.formValueWithDefault (userDateToString asset.userDate) form "user_date"
        tags =
            Forms.formValueWithDefault (String.join ", " asset.tags) form "tags"
    in
        -- Apparently the "on submit" on the form works better than using "on
        -- click" on a particular form input/button.
        Html.form [ onSubmit (SubmitAsset asset.id) ]
            [ div [ class "container", style [("width", "auto"), ("padding-right", "3em")] ]
                [ editAssetFormGroup form "user_date" "Custom Date" userDate "text" "yyyy-mm-dd HH:MM"
                , editAssetFormGroup form "location" "Location" location "text" ""
                , editAssetFormGroup form "caption" "Caption" caption "text" ""
                , editAssetFormGroup form "tags" "Tags" tags "text" "(comma-separated)"
                , div [ class "field is-horizontal" ]
                    [ div [ class "field-label" ] [ ]
                    , div [ class "field-body" ] [ assetEditSaveButton form asset ]
                    ]
                ]
            ]


editAssetFormGroup : Forms.Form -> String -> String -> String -> String -> String -> Html Msg
editAssetFormGroup form idString labelText value inputType placeholderText =
    let
        validateMsg =
            Forms.errorString form idString
        formIsValid =
            validateMsg == "no errors"
        inputClass =
            if formIsValid then
                "input"
            else
                "input is-danger"
        inputField =
            div [ class "control" ]
                [ input
                    [ id idString
                    , class inputClass
                    , type_ inputType
                    , Html.Attributes.name idString
                    , Html.Attributes.value value
                    , onInput (UpdateFormAssetEdit idString)
                    , placeholder placeholderText
                    ] [ ]
                ]
        validationTextDiv =
            p [ class "help is-danger" ] [ text validateMsg ]
        formGroupElems =
            if formIsValid then
                [ inputField ]
            else
                [ inputField, validationTextDiv ]
    in
        div [ class "field is-horizontal" ]
            [ div [ class "field-label is-normal" ]
                [ Html.label [ for idString, class "label" ] [ text labelText ] ]
            , div [ class "field-body" ]
                [ div [ class "field" ] formGroupElems ]
            ]


assetEditSaveButton : Forms.Form -> AssetDetails -> Html Msg
assetEditSaveButton form asset =
    let
        attrs =
            if Forms.validateStatus form then
                [ type_ "submit", value "Save", class "button is-primary" ]
            else
                [ type_ "submit", value "Save", class "button", disabled True ]
    in
        input attrs [ ]
