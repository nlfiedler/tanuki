module View exposing (..)

import Forms
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (onClick, onInput, onSubmit)
import Html.Keyed
import Json.Encode exposing (string)
import List.Extra exposing (greedyGroupsOf)
import Messages exposing (..)
import Model exposing (..)
import RemoteData exposing (WebData)
import Routing exposing (Route(..))


view : Model -> Html Msg
view model =
    case model.route of
        HomeIndexRoute ->
            indexPage model

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


tagSelector : Model -> Html Msg
tagSelector model =
    let
        header =
            li [ ] [ strong [ ] [ text "Tags:" ] ]
        entries =
            header :: viewTagList ToggleTag model.tagList
    in
        ul [ class "list-inline" ] entries


viewTagList : (String -> msg) -> WebData TagList -> List (Html msg)
viewTagList msg entries =
    case entries of
        RemoteData.NotAsked ->
            [ text "Initializing..." ]

        RemoteData.Loading ->
            [ text "Loading tags..." ]

        RemoteData.Failure error ->
            [ text (toString error) ]

        RemoteData.Success list ->
            List.map (viewTagItem msg) list


viewTagItem : (String -> msg) -> Tag -> Html msg
viewTagItem msg entry =
    let
        linkBody =
            if entry.selected then
                strong [ ] [ text entry.label ]
            else
                text entry.label
    in
        li [ ]
            [ a [ href "#", onClick (msg entry.label) ] [ linkBody ] ]


yearSelector : Model -> Html Msg
yearSelector model =
    let
        header =
            li [ ] [ strong [ ] [ text "Years:" ] ]
        entries =
            header :: viewYearList ToggleYear model.yearList
    in
        ul [ class "list-inline" ] entries


viewYearList : (Int -> msg) -> WebData YearList -> List (Html msg)
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
        linkBody =
            if entry.selected then
                strong [ ] [ text (toString entry.year) ]
            else
                text (toString entry.year)
    in
        li [ ]
            [ a [ href "#", onClick (msg entry.year) ] [ linkBody ] ]


locationSelector : Model -> Html Msg
locationSelector model =
    let
        header =
            li [ ] [ strong [ ] [ text "Locations:" ] ]
        entries =
            header :: viewLocationList ToggleLocation model.locationList
    in
        ul [ class "list-inline" ] entries


viewLocationList : (String -> msg) -> WebData LocationList -> List (Html msg)
viewLocationList msg entries =
    case entries of
        RemoteData.NotAsked ->
            [ text "Initializing..." ]

        RemoteData.Loading ->
            [ text "Loading locations..." ]

        RemoteData.Failure error ->
            [ text (toString error) ]

        RemoteData.Success list ->
            List.map (viewLocationItem msg) list


viewLocationItem : (String -> msg) -> Location -> Html msg
viewLocationItem msg entry =
    let
        linkBody =
            if entry.selected then
                strong [ ] [ text entry.label ]
            else
                text entry.label
    in
        li [ ]
            [ a [ href "#", onClick (msg entry.label) ] [ linkBody ] ]


viewThumbnails : Model -> Html Msg
viewThumbnails model =
    case model.assetList of
        RemoteData.NotAsked ->
            text "Make a selection above to display assets."

        RemoteData.Loading ->
            text "Loading thumbnails..."

        RemoteData.Failure error ->
            text (toString error)

        RemoteData.Success list ->
            let
                cells =
                    List.map viewThumbnailItem list.entries
                groups =
                    greedyGroupsOf 3 cells
                rows =
                    List.map (div [ class "row" ]) groups
                paging =
                    -- skip the pagination links if there is nothing to page
                    if list.total_entries > pageSize && List.length rows > 1 then
                        paginationList model.pageNumber list
                    else
                        [ text "" ]
            in
                div [ ] (rows ++ paging)


viewThumbnailItem : AssetSummary -> Html Msg
viewThumbnailItem entry =
    let
        -- trick to put in a nice separator for the thumbnail caption
        separator =
            span [ property "innerHTML" <| string "&mdash;" ] [ ]
    in
        -- The images are likely being resized to fit the container, but
        -- for now they look okay.
        div [ class "col-sm-6 col-md-4" ]
            [ div [ class "thumbnail"
                  , onClick <| NavigateTo <| ShowAssetRoute entry.id
                  ]
                [ img [ src ("/thumbnail/" ++ entry.checksum)
                      , alt entry.filename
                      ] [ ]
                , div [ class "caption" ]
                    [ text entry.date
                    , separator
                    , text entry.filename ]
                ]
            ]


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
        preDots =
            if lower > 2 then
                Just ( ",...", li [ class "disabled" ] [ span [ ] [ text "..." ] ] )
            else
                Nothing
        postDots =
            if upper < (totalPages - 1) then
                Just ( "...,", li [ class "disabled" ] [ span [ ] [ text "..." ] ] )
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
            Html.Keyed.ul [ class "pagination" ] (justLinksExtractor maybeLinks)
    in
        [nav [ class "text-center" ] [ linkList ] ]


namedPaginationLink : String -> Int -> Maybe ( String, Html Msg )
namedPaginationLink label page =
    Just ( toString page, li [ ] [ a [ onClick <| Paginate page ] [ text label ] ] )


paginationLink : Int -> Int -> Maybe ( String, Html Msg )
paginationLink currentPage page =
    let
        classes =
            classList [ ( "active", currentPage == page ) ]
    in
        Just ( toString page
            , li [ classes ]
                [ a [ onClick <| Paginate page ] [ text (toString page) ] ]
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
            div [ ]
                [ (viewAssetPreviewPanel asset)
                , dl [ class "dl-horizontal" ] (viewAssetDetails asset)
                , div [ class "col-sm-offset-2 col-sm-10" ]
                    [ a [ class "btn btn-default"
                        , onClick <| NavigateTo <| EditAssetRoute asset.id ] [ text "Edit" ]
                    ]
                , backToHomeLink
                ]


viewAssetPreviewPanel : AssetDetails -> Html Msg
viewAssetPreviewPanel asset =
    div [ class "panel panel-default" ]
        [ div [ class "panel-heading" ]
            [ h3 [ class "panel-title" ] [ text asset.filename ] ]
        , div [ class "panel-body" ] [ viewAssetPreview asset ]
        , div [ class "panel-footer" ] [ text asset.datetime ]
        ]


viewAssetPreview : AssetDetails -> Html Msg
viewAssetPreview asset =
    if String.startsWith "video/" asset.mimetype then
        video [ style [ ("width", "100%"), ("height", "100%") ]
              , controls True, preload "auto" ]
            [ source [ src ("/asset/" ++ asset.checksum)
                     , type_ (assetMimeType asset.mimetype) ] [ ]
            , text "Bummer, your browser does not support the HTML5"
            , code [ ] [ text "video" ]
            , text "tag."
            ]
    else
        a [ href ("/asset/" ++ asset.checksum) ]
            [ img [ class "asset"
                  , src ("/preview/" ++ asset.checksum)
                  , alt asset.filename ] [ ]
            ]


viewAssetDetails : AssetDetails -> List (Html Msg)
viewAssetDetails asset =
    let
        part1 =
            [ ( "Size", (toString asset.size) )
            , ( "SHA256", asset.checksum )
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
        dt_dd ( t, d ) =
            [ dt [ ] [ text t ]
            , dd [ ] [ text d ]
            ]
    in
        List.concatMap (dt_dd) rows


notFoundView : Html Msg
notFoundView =
    warningMessage "Page not found" backToHomeLink


warningMessage : String -> Html Msg -> Html Msg
warningMessage message content =
    div [ class "alert alert-warning" ]
        [ text message
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
            div [ ]
                [ (viewAssetPreviewPanel asset)
                , editAssetForm model.assetEditForm asset
                , backToHomeLink
                ]


editAssetForm : Forms.Form -> AssetDetails -> Html Msg
editAssetForm form asset =
    let
        location =
            Forms.formValueWithDefault (Maybe.withDefault "" asset.location) form "location"
        caption =
            Forms.formValueWithDefault (Maybe.withDefault "" asset.caption) form "caption"
        userDate =
            Forms.formValueWithDefault (extractUserDate asset) form "user_date"
        tags =
            Forms.formValueWithDefault (String.join ", " asset.tags) form "tags"
    in
        -- Apparently the "on submit" on the form works better than using "on
        -- click" on a particular form input/button.
        Html.form [ class "form-horizontal"
                  , onSubmit (SubmitAsset asset.id)
                  ]
            [ div [ class "form-group" ]
                [ editAssetFormGroup form "user_date" "Custom Date" userDate "date" "yyyy-mm-dd"
                , editAssetFormGroup form "location" "Location" location "text" ""
                , editAssetFormGroup form "caption" "Caption" caption "text" ""
                , editAssetFormGroup form "tags" "Tags" tags "text" "(comma-separated)"
                , div [ class "form-group" ]
                    [ div [ class "col-sm-offset-2 col-sm-10" ]
                        [ assetEditSaveButton form asset ]
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
        formGroupClass =
            if formIsValid then
                "form-group"
            else
                "form-group has-error"
        inputField =
            input
                [ id idString
                , class "form-control"
                , type_ inputType
                , Html.Attributes.name idString
                , Html.Attributes.value value
                , onInput (UpdateFormAssetEdit idString)
                , placeholder placeholderText
                ] [ ]
        validationTextDiv =
            div [ class "help-block" ] [ text (Forms.errorString form idString) ]
        formGroupElems =
            if formIsValid then
                [ inputField ]
            else
                [ inputField, validationTextDiv ]
    in
        div [ class formGroupClass ]
            [ Html.label [ for idString, class "col-sm-2 control-label" ] [ text labelText ]
            , div [ class "col-sm-10" ] formGroupElems
            ]


assetEditSaveButton : Forms.Form -> AssetDetails -> Html Msg
assetEditSaveButton form asset =
    let
        attrs =
            if Forms.validateStatus form then
                [ type_ "submit", value "Save", class "btn btn-default" ]
            else
                [ type_ "submit", value "Save", class "btn", disabled True ]
    in
        input attrs [ ]