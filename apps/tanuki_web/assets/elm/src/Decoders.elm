module Decoders exposing (..)

import Json.Decode exposing (int, list, nullable, string, Decoder)
import Json.Decode.Pipeline exposing (decode, hardcoded, required)
import Model exposing (..)


tagDecoder : Decoder Tag
tagDecoder =
    decode Tag
        |> required "tag" string
        |> required "count" int
        |> hardcoded False


yearDecoder : Decoder Year
yearDecoder =
    decode Year
        |> required "year" int
        |> required "count" int
        |> hardcoded False


locationDecoder : Decoder Location
locationDecoder =
    decode Location
        |> required "location" string
        |> required "count" int
        |> hardcoded False


-- Decode the response to the assets query, which includes a list of
-- summary information and a total count.
assetsDecoder : Decoder AssetList
assetsDecoder =
    let
        -- decoder for an assets query entry
        entry =
            decode AssetSummary
                |> required "id" string
                |> required "filename" string
                |> required "date" string
                |> required "checksum" string
    in
        decode AssetList
            |> required "count" int
            |> required "assets" (list entry)


-- Decode all of the details of a single asset.
assetDecoder : Decoder AssetDetails
assetDecoder =
    decode AssetDetails
        |> required "id" string
        |> required "filename" string
        |> required "size" int
        |> required "mimetype" string
        |> required "datetime" string
        |> required "user_date" (nullable string)
        |> required "checksum" string
        |> required "caption" (nullable string)
        |> required "location" (nullable string)
        |> required "duration" (nullable int)
        |> required "tags" (list string)
