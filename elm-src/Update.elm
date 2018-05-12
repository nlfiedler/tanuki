module Update exposing (..)

import Commands exposing (..)
import Forms
import Messages exposing (..)
import Model exposing (..)
import Navigation
import RemoteData exposing (WebData)
import Routing exposing (Route(..), parse, toPath)


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        TagsResponse response ->
            ( { model | tagList = unwrapResponse response }, Cmd.none )

        YearsResponse response ->
            ( { model | yearList = unwrapResponse response }, Cmd.none )

        LocationsResponse response ->
            ( { model | locationList = unwrapResponse response }, Cmd.none )

        ToggleTag label ->
            let
                -- When tag selection changes, request page 1 as the
                -- current page almost certainly does not make sense now.
                ( updatedTags, cmd ) =
                    RemoteData.update (updateTagSelection model label 1) model.tagList
            in
                -- Reset the page number when the query has changed.
                ( { model | tagList = updatedTags, pageNumber = 1 }, cmd )

        ToggleYear year ->
            let
                -- When year selection changes, request page 1 as the
                -- current page almost certainly does not make sense now.
                ( updatedYears, cmd ) =
                    RemoteData.update (updateYearSelection model year 1) model.yearList
            in
                -- Reset the page number when the query has changed.
                ( { model | yearList = updatedYears, pageNumber = 1 }, cmd )

        ToggleLocation label ->
            let
                -- When location selection changes, request page 1 as the
                -- current page almost certainly does not make sense now.
                ( updatedLocations, cmd ) =
                    RemoteData.update (updateLocationSelection model label 1) model.locationList
            in
                -- Reset the page number when the query has changed.
                ( { model | locationList = updatedLocations, pageNumber = 1 }, cmd )

        ToggleAllTags ->
            ( { model | showingAllTags = not model.showingAllTags }, Cmd.none )

        ToggleAllLocations ->
            ( { model | showingAllLocations = not model.showingAllLocations }, Cmd.none )

        AssetsResponse response ->
            ( { model | assetList = unwrapResponse response }, Cmd.none )

        ThumblessAsset id ->
            ( { model | assetList = markThumbless model.assetList id }, Cmd.none )

        Paginate pageNumber ->
            -- We need the tags in order to update the page selection, but
            -- otherwise the updated list is not used.
            let
                ( updatedTags, cmd ) =
                    RemoteData.update (updatePageSelection model pageNumber) model.tagList
            in
                ( { model | pageNumber = pageNumber }, cmd )

        AssetResponse response ->
            ( { model |
                  asset = unwrapResponse response,
                  assetEditForm = updateAssetEditForm (unwrapResponse response)
              }, Cmd.none )

        UpdateFormAssetEdit fieldName value ->
            -- A single value changed on the asset edit form, which typically
            -- means a single character was added or removed. We validate the
            -- input and enable or disable the Save button.
            let
                newForm =
                    Forms.updateFormInput model.assetEditForm fieldName value
            in
                ( { model | assetEditForm = newForm }, Cmd.none )

        SubmitAsset assetId ->
            -- The asset id is expected simply for convenience.
            ( model, updateAsset assetId model )

        SubmitResponse assetId response ->
            -- The asset id is expected simply for convenience.
            let
                navCmd =
                    Navigation.newUrl <| toPath (ShowAssetRoute assetId)
            in
                -- receive and update the asset model as the backend may have
                -- changed some values from what was submitted
                ( { model | asset = unwrapResponse response }
                -- refresh attribute lists in case of attribute changes
                , Cmd.batch [sendTagsQuery, sendYearsQuery, sendLocationsQuery, navCmd]
                )

        UploadSelection filename ->
            let
                basename =
                    -- The browser adds a fake path onto the file name; this is
                    -- only for viewing purposes, the upload form will take care
                    -- of reading and uploading the file.
                    if String.startsWith "C:\\fakepath\\" filename then
                        String.dropLeft 12 filename
                    else
                        filename
            in
                ( { model | uploadFilename = Just basename }, Cmd.none )

        UrlChange location ->
            let
                currentRoute =
                    parse location
            in
                urlUpdate { model | route = currentRoute }

        NavigateTo route ->
            ( model, Navigation.newUrl <| toPath route )


-- Invoked via RemoteData.update() to update the tag list and construct a
-- command to fetch the matching assets.
updateTagSelection : Model -> String -> Int -> TagList -> ( TagList, Cmd Msg )
updateTagSelection model label page tags =
    let
        toggleEntry e =
            if e.label == label then
                { e | selected = (not e.selected) }
            else
                e
        updatedTags =
            List.map toggleEntry tags
        selectedTags =
            List.filter (.selected) updatedTags
        selectedYears =
            getSelectedYears model
        selectedLocations =
            getSelectedLocations model
    in
        -- empty selection is okay
        ( updatedTags, sendAssetsQuery page selectedTags selectedYears selectedLocations )


-- Invoked via RemoteData.update() to update the year list and construct a
-- command to fetch the matching assets.
updateYearSelection : Model -> Int -> Int -> YearList -> ( YearList, Cmd Msg )
updateYearSelection model year page years =
    let
        toggleEntry e =
            if e.year == year then
                { e | selected = (not e.selected) }
            else
                e
        updatedYears =
            List.map toggleEntry years
        selectedTags =
            getSelectedTags model
        selectedYears =
            List.filter (.selected) updatedYears
        selectedLocations =
            getSelectedLocations model
    in
        -- empty selection is okay
        ( updatedYears, sendAssetsQuery page selectedTags selectedYears selectedLocations )


-- Invoked via RemoteData.update() to update the location list and
-- construct a command to fetch the matching assets.
updateLocationSelection : Model -> String -> Int -> LocationList -> ( LocationList, Cmd Msg )
updateLocationSelection model location page locations =
    let
        toggleEntry e =
            if e.label == location then
                { e | selected = (not e.selected) }
            else
                e
        updatedLocations =
            List.map toggleEntry locations
        selectedTags =
            getSelectedTags model
        selectedYears =
            getSelectedYears model
        selectedLocations =
            List.filter (.selected) updatedLocations
    in
        -- empty selection is okay
        ( updatedLocations, sendAssetsQuery page selectedTags selectedYears selectedLocations )


-- Invoked via RemoteData.update() to construct a command to fetch the
-- matching assets for the given page.
updatePageSelection : Model -> Int -> TagList -> ( TagList, Cmd Msg )
updatePageSelection model page tags =
    let
        selectedTags =
            List.filter (.selected) tags
        selectedYears =
            getSelectedYears model
        selectedLocations =
            getSelectedLocations model
    in
        -- empty selection is okay
        ( tags, sendAssetsQuery page selectedTags selectedYears selectedLocations )


urlUpdate : Model -> ( Model, Cmd Msg )
urlUpdate model =
    case model.route of
        HomeIndexRoute ->
            ( model, refreshModelCommands model )

        ShowAssetRoute id ->
            ( { model | asset = RemoteData.Loading }, fetchAsset id )

        EditAssetRoute id ->
            -- The Elixir-based upload page jumps directly to this URL, so we
            -- need to fetch everything from scratch. It also serves to
            -- refresh the asset model, in case of concurrent edits.
            ( { model | asset = RemoteData.Loading }, fetchAsset id )

        _ ->
            ( model, Cmd.none )


{- Return the selected tags.

May return an empty list if the tag list has not been requested, or is
otherwise not available at this time.

-}
getSelectedTags : Model -> TagList
getSelectedTags model =
    let
        tags =
            RemoteData.withDefault [] model.tagList
    in
        List.filter (.selected) tags


{- Return the selected years.

May return an empty list if the year list has not been requested, or is
otherwise not available at this time.

-}
getSelectedYears : Model -> YearList
getSelectedYears model =
    let
        years =
            RemoteData.withDefault [] model.yearList
    in
        List.filter (.selected) years


{- Return the selected locations.

May return an empty list if the location list has not been requested, or is
otherwise not available at this time.

-}
getSelectedLocations : Model -> LocationList
getSelectedLocations model =
    let
        locations =
            RemoteData.withDefault [] model.locationList
    in
        List.filter (.selected) locations


{- Generate commands to fill in the missing pieces of the model
-}
refreshModelCommands : Model -> Cmd Msg
refreshModelCommands model =
    let
        cmd1 =
            if RemoteData.isSuccess model.tagList then
                Cmd.none
            else
                sendTagsQuery
        cmd2 =
            if RemoteData.isSuccess model.yearList then
                Cmd.none
            else
                sendYearsQuery
        cmd3 =
            if RemoteData.isSuccess model.locationList then
                Cmd.none
            else
                sendLocationsQuery
    in
        Cmd.batch [cmd1, cmd2, cmd3]


{- Ensure the asset edit form fields are populated with values from the model.
-}
updateAssetEditForm : GraphData AssetDetails -> Forms.Form
updateAssetEditForm response =
    case response of
        RemoteData.NotAsked ->
            initialAssetEditForm
        RemoteData.Loading ->
            initialAssetEditForm
        RemoteData.Failure err ->
            initialAssetEditForm
        RemoteData.Success asset ->
            let
                form1 =
                    Forms.updateFormInput initialAssetEditForm "user_date" (userDateToString asset.userDate)
                form2 =
                    Forms.updateFormInput form1 "location" (Maybe.withDefault "" asset.location)
                form3 =
                    Forms.updateFormInput form2 "caption" (Maybe.withDefault "" asset.caption)
                finalForm =
                    Forms.updateFormInput form3 "tags" (String.join ", " asset.tags)
            in
                finalForm


{- Mark the asset with the given identifier as missing its thumbnail.
-}
markThumbless : GraphData AssetList -> String -> GraphData AssetList
markThumbless assetList id =
    let
        thumbmarker asset =
            if asset.id == id && asset.thumbless == False then
                { asset | thumbless = True}
            else
                asset
        processList list =
            { list | entries = List.map thumbmarker list.entries}
    in
        RemoteData.map processList assetList
