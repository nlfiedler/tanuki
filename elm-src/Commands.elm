module Commands exposing (..)

import Decoders exposing (..)
import Forms
import Http
import Json.Decode as Decode
import Json.Encode as Encode
import Messages exposing (Msg(..))
import Model exposing (..)
import RemoteData exposing (WebData)


apiUrlPrefix : String
apiUrlPrefix =
    "http://localhost:3000/api"


getTags : Cmd Msg
getTags =
    Decode.list tagDecoder
        |> Http.get (apiUrlPrefix ++ "/tags")
        |> RemoteData.sendRequest
        |> Cmd.map FetchTags


getYears : Cmd Msg
getYears =
    Decode.list yearDecoder
        |> Http.get (apiUrlPrefix ++ "/years")
        |> RemoteData.sendRequest
        |> Cmd.map FetchYears


getLocations : Cmd Msg
getLocations =
    Decode.list locationDecoder
        |> Http.get (apiUrlPrefix ++ "/locations")
        |> RemoteData.sendRequest
        |> Cmd.map FetchLocations


{- Request the assets that match the given criteria.

The tags can be an empty list, which results in a response that contains
nothing more than total count of all assets. Any combination of tags,
years, and locations can be used to query assets.

-}
getAssets : Int -> TagList -> YearList -> LocationList -> Cmd Msg
getAssets page tags years locations =
    let
        tag_params =
            List.map (\t -> "tags[]=" ++ t.label) tags
        year_params =
            List.map (\y -> "years[]=" ++ (toString y.year)) years
        location_params =
            List.map (\l -> "locations[]=" ++ l.label) locations
        page_params =
            [ "page=" ++ (toString page), "page_size=" ++ (toString pageSize) ]
        params =
            String.join "&" (tag_params ++ year_params ++ location_params ++ page_params)
    in
        assetsDecoder
            |> Http.get (apiUrlPrefix ++ "/assets?" ++ params)
            |> RemoteData.sendRequest
            |> Cmd.map QueryAssets


fetchAsset : String -> Cmd Msg
fetchAsset id =
    assetDecoder
        |> Http.get (apiUrlPrefix ++ "/assets/" ++ id)
        |> RemoteData.sendRequest
        |> Cmd.map FetchAsset


updateAsset : String -> Model -> Cmd Msg
updateAsset id model =
    let
        attributes =
            [ ("location", Encode.string (Forms.formValue model.assetEditForm "location"))
            , ("caption", Encode.string (Forms.formValue model.assetEditForm "caption"))
            , ("tags", Encode.string (Forms.formValue model.assetEditForm "tags"))
            , ("user_date", Encode.string (Forms.formValue model.assetEditForm "user_date"))
            ]
        encodedBody =
            Encode.object attributes
    in
        -- The decoder is not really used here, as the response from the
        -- backend is nothing more than a "status" message. But we need to do
        -- something with the request and convert it to a command.
        Http.request
                { method = "PUT"
                , headers = []
                , url = (apiUrlPrefix ++ "/assets/" ++ id)
                , body = encodedBody |> Http.jsonBody
                , expect = Http.expectJson assetDecoder
                , timeout = Nothing
                , withCredentials = False
                }
            |> RemoteData.sendRequest
            |> Cmd.map (PostSubmitAsset id)
