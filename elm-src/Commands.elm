module Commands exposing (..)
{-| The GraphQL client code for fetching data from the backend.

Both the decoders and the query logic is implemented here.

-}

import Forms
import GraphQL.Request.Builder exposing (..)
import GraphQL.Request.Builder.Arg as Arg
import GraphQL.Request.Builder.Variable as Var
import GraphQL.Client.Http as GraphQLClient
import Messages exposing (Msg(..))
import Model exposing (..)
import RemoteData exposing (WebData)
import Task exposing (Task)


graphqlEndpoint : String
graphqlEndpoint =
    "http://localhost:3000/graphql"


{-| GraphQL specification for a single TagCount object.
-}
tagSpec : ValueSpec NonNull ObjectType Tag vars
tagSpec =
    object Tag
        |> with (field "value" [] string)
        |> with (field "count" [] int)
        |> withLocalConstant False


{-| GraphQL specification for a single YearCount object.
-}
yearSpec : ValueSpec NonNull ObjectType Year vars
yearSpec =
    object Year
        |> with (field "value" [] int)
        |> with (field "count" [] int)
        |> withLocalConstant False


{-| GraphQL specification for a single LocationCount object.
-}
locationSpec : ValueSpec NonNull ObjectType Location vars
locationSpec =
    object Location
        |> with (field "value" [] string)
        |> with (field "count" [] int)
        |> withLocalConstant False


{-| GraphQL specification for a single search result object.
-}
summarySpec : ValueSpec NonNull ObjectType AssetSummary vars
summarySpec =
    object AssetSummary
        |> with (field "id" [] id)
        |> with (field "datetime" [] int)
        |> with (field "filename" [] string)
        |> with (field "location" [] (nullable string))
        |> withLocalConstant False


{-| GraphQL specification for the results of a search.
-}
searchSpec : ValueSpec NonNull ObjectType AssetList vars
searchSpec =
    object AssetList
        |> with (field "count" [] int)
        |> with (field "results" [] (list summarySpec))


{-| GraphQL specification for an asset.
-}
assetSpec :  ValueSpec NonNull ObjectType AssetDetails vars
assetSpec =
    object AssetDetails
        |> with (field "id" [] string)
        |> with (field "filename" [] string)
        |> with (field "filesize" [] int)
        |> with (field "mimetype" [] string)
        |> with (field "datetime" [] int)
        |> with (field "userdate" [] (nullable int))
        |> with (field "caption" [] (nullable string))
        |> with (field "location" [] (nullable string))
        |> with (field "duration" [] (nullable float))
        |> with (field "tags" [] (list string))


{-| Convenience function for sending a GraphQL query.
-}
sendQueryRequest : Request Query a -> Task GraphQLClient.Error a
sendQueryRequest request =
    GraphQLClient.sendQuery graphqlEndpoint request


{-| Convenience function for sending a GraphQL mutation.
-}
sendMutationRequest : Request Mutation a -> Task GraphQLClient.Error a
sendMutationRequest request =
    GraphQLClient.sendMutation graphqlEndpoint request


{-| Convert a GraphQL query result to a RemoteData object.

Extract the result from the GraphQL query and convert it to the RemoteData
object that is far more useful than a dumb Result.

-}
unwrapResponse : GraphResult a -> (RemoteData.RemoteData GraphQLClient.Error a)
unwrapResponse response =
    case response of
        Ok value ->
            RemoteData.Success value
        Err error ->
            RemoteData.Failure error


{-| Send a request for the tags and their counts.
-}
sendTagsQuery : Cmd Msg
sendTagsQuery =
    let
        tagsRequest =
            extract
                (field "tags" [] (list tagSpec))
                |> queryDocument
                |> request {}
    in
        sendQueryRequest tagsRequest
            |> Task.attempt TagsResponse


{-| Send a request for the years and their counts.
-}
sendYearsQuery : Cmd Msg
sendYearsQuery =
    let
        yearsRequest =
            extract
                (field "years" [] (list yearSpec))
                |> queryDocument
                |> request {}
    in
        sendQueryRequest yearsRequest
            |> Task.attempt YearsResponse


{-| Send a request for the locations and their counts.
-}
sendLocationsQuery : Cmd Msg
sendLocationsQuery =
    let
        locationsRequest =
            extract
                (field "locations" [] (list locationSpec))
                |> queryDocument
                |> request {}
    in
        sendQueryRequest locationsRequest
            |> Task.attempt LocationsResponse


{- Request the assets that match the given criteria.

Any combination of tags, years, and locations can be used to query assets.

-}
sendAssetsQuery : Int -> TagList -> YearList -> LocationList -> Cmd Msg
sendAssetsQuery page tags years locations =
    let
        tagsVar =
            Var.optional "tags" .tags (Var.list Var.string) []
        locationsVar =
            Var.optional "locations" .locations (Var.list Var.string) []
        yearsVar =
            Var.optional "years" .years (Var.list Var.int) []
        countVar =
            Var.optional "count" .count Var.int 10
        offsetVar =
            Var.optional "offset" .offset Var.int 0
        assetsRequest =
            extract
                (field "search"
                    [ ("tags", Arg.variable tagsVar)
                    , ("locations", Arg.variable locationsVar)
                    , ("years", Arg.variable yearsVar)
                    , ("count", Arg.variable countVar)
                    , ("offset", Arg.variable offsetVar)
                    ]
                    searchSpec
                )
                |> queryDocument
                |> request
                    { tags = Just (List.map .label tags)
                    , locations = Just (List.map .label locations)
                    , years = Just (List.map .year years)
                    , count = Just pageSize
                    , offset = Just ((page - 1) * pageSize)
                    }
    in
        sendQueryRequest assetsRequest
            |> Task.attempt AssetsResponse


{-| Fetch the details for an asset using the identifier.
-}
fetchAsset : String -> Cmd Msg
fetchAsset id =
    let
        idVar =
            Var.required "id" .id Var.id
        assetRequest =
            extract
                (field "asset"
                    [ ("id", Arg.variable idVar) ]
                    assetSpec
                )
                |> queryDocument
                |> request { id = id }
    in
        sendQueryRequest assetRequest
            |> Task.attempt AssetResponse


{-| Update some of the details for an asset.
-}
updateAsset : String -> Model -> Cmd Msg
updateAsset id model =
    let
        splitTags =
            String.split "," (Forms.formValue model.assetEditForm "tags")
        tagsList =
            List.map (\t -> String.trim t) splitTags
        idVar =
            Var.required "id" .id Var.id
        assetVar =
            Var.required "asset" .asset
                (Var.object "AssetInput"
                    [ Var.field "tags" .tags (Var.list Var.string)
                    , Var.field "caption" .caption Var.string
                    , Var.field "location" .location Var.string
                    -- allow a nullable datetime so we can erase the value
                    , Var.optionalField "datetime" .datetime (Var.nullable Var.int)
                    ]
                )
        updateRequest =
            extract
                (field "update"
                    [ ("id", Arg.variable idVar)
                    , ("asset", Arg.variable assetVar)
                    ]
                    assetSpec
                )
                |> mutationDocument
                |> request
                    { id = id
                    , asset =
                        { tags = tagsList
                        , caption = (Forms.formValue model.assetEditForm "caption")
                        , location = (Forms.formValue model.assetEditForm "location")
                        -- double-wrap the user date so we can send null values,
                        -- otherwise we cannot remove the previously set value
                        , datetime = Just (userDateStrToInt (Forms.formValue model.assetEditForm "user_date"))
                        }
                    }
    in
        sendMutationRequest updateRequest
            |> Task.attempt (SubmitResponse id)
