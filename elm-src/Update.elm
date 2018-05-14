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
                newModel =
                    updateTagSelection model label
            in
                ( newModel, fetchAssets newModel )

        ToggleYear year ->
            let
                newModel =
                    updateYearSelection model year
            in
                ( newModel, fetchAssets newModel )

        ToggleLocation label ->
            let
                newModel =
                    updateLocationSelection model label
            in
                ( newModel, fetchAssets newModel )

        ToggleAllTags ->
            ( { model | showingAllTags = not model.showingAllTags }, Cmd.none )

        ToggleAllLocations ->
            ( { model | showingAllLocations = not model.showingAllLocations }, Cmd.none )

        AssetsResponse response ->
            ( { model | assetList = unwrapResponse response }, Cmd.none )

        ThumblessAsset id ->
            ( { model | assetList = markThumbless model.assetList id }, Cmd.none )

        Paginate page ->
            let
                newModel =
                    { model | pageNumber = page }
            in
                ( newModel, fetchAssets newModel )

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

        UpdateFormSearch fieldName value ->
            -- A single value changed on the search form, which typically
            -- means a single character was added or removed.
            let
                newForm =
                    Forms.updateFormInput model.assetSearchForm fieldName value
                savedSearch =
                    saveSearchValues newForm
            in
                ( { model | assetSearchForm = newForm, savedSearch = savedSearch }, Cmd.none )

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

        SearchAssets ->
            -- Reset the page number when the query has changed.
            let
                newModel =
                    { model | pageNumber = 1 }
            in
                ( newModel, fetchAssets newModel )

        UrlChange location ->
            let
                currentRoute =
                    parse location
            in
                urlUpdate { model | route = currentRoute }

        NavigateTo route ->
            ( model, Navigation.newUrl <| toPath route )


{-| Update the model to reflect the change in tag selection.
-}
updateTagSelection : Model -> String -> Model
updateTagSelection model label =
    let
        toggleEntry e =
            if e.label == label then
                { e | selected = (not e.selected) }
            else
                e
        updateList l =
            ( List.map toggleEntry l, Cmd.none )
        ( updatedTags, cmd ) =
            RemoteData.update updateList model.tagList
        newModel =
            { model | pageNumber = 1, tagList = updatedTags }
    in
        newModel


{-| Update the model to reflect the change in year selection.
-}
updateYearSelection : Model -> Int -> Model
updateYearSelection model year =
    let
        toggleEntry e =
            if e.year == year then
                { e | selected = (not e.selected) }
            else
                e
        updateList l =
            ( List.map toggleEntry l, Cmd.none )
        ( updatedYears, cmd ) =
            RemoteData.update updateList model.yearList
        newModel =
            { model | pageNumber = 1, yearList = updatedYears }
    in
        newModel


{-| Update the model to reflect the change in year selection.
-}
updateLocationSelection : Model -> String -> Model
updateLocationSelection model location =
    let
        toggleEntry e =
            if e.label == location then
                { e | selected = (not e.selected) }
            else
                e
        updateList l =
            ( List.map toggleEntry l, Cmd.none )
        ( updatedLocations, cmd ) =
            RemoteData.update updateList model.locationList
        newModel =
            { model | pageNumber = 1, locationList = updatedLocations }
    in
        newModel


urlUpdate : Model -> ( Model, Cmd Msg )
urlUpdate model =
    case model.route of
        HomeIndexRoute ->
            ( model, refreshModelCommands model )

        ShowAssetRoute id ->
            ( { model | asset = RemoteData.Loading }, fetchAsset id )

        EditAssetRoute id ->
            -- The backend redirects to this URL after an upload has completed,
            -- and since it likely added additional details to the asset, we
            -- fetch the values here.
            ( { model | asset = RemoteData.Loading }, fetchAsset id )

        _ ->
            ( model, Cmd.none )


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


{-| Create a new saved search record based on the updated form.
-}
saveSearchValues : Forms.Form -> SearchValues
saveSearchValues form =
    { tags = Forms.formValue form "tags"
    , locations = Forms.formValue form "locations"
    , before = Forms.formValue form "before"
    , after = Forms.formValue form "after"
    , filename = Forms.formValue form "filename"
    , mimetype = Forms.formValue form "mimetype"
    }


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
