module Model exposing (..)

import Date
import Forms
import GraphQL.Client.Http as GraphQLClient
import Regex
import RemoteData
import Routing exposing (Route)
import Time


-- Instead of RemoteData.WebData, we define our own data type that represents
-- the response from the GraphQL endpoint, with a GraphQL error type.
type alias GraphData a =
    RemoteData.RemoteData GraphQLClient.Error a


-- Convenience for the thing returned from a GraphQL query.
type alias GraphResult a = Result GraphQLClient.Error a


type alias Model =
    { tagList : GraphData TagList
    , yearList : GraphData YearList
    , locationList : GraphData LocationList
    , assetList : GraphData AssetList
    , pageNumber : Int
    , asset : GraphData AssetDetails
    , route : Route
    , assetEditForm : Forms.Form
    , showingAllTags : Bool
    , showingAllLocations : Bool
    }


type alias Tag =
    { label : String
    , count : Int
    , selected : Bool
    }


type alias TagList = (List Tag)


type alias Year =
    { year : Int
    , count : Int
    , selected : Bool
    }


type alias YearList = (List Year)


type alias Location =
    { label : String
    , count : Int
    , selected : Bool
    }


type alias LocationList = (List Location)


-- A subset of all assets, hence total_entries to indicate just how many
-- assets there are that match the selection criteria.
type alias AssetList =
    { total_entries : Int
    , entries : List AssetSummary
    }


-- The information returned from an assets query.
type alias AssetSummary =
    { id : String
    , date : Int
    , file_name : String
    , location : Maybe String
    , thumbless : Bool  -- if True, thumbnail request had an error
    }


-- Detailed information on a single asset.
type alias AssetDetails =
    { id : String
    , file_name : String
    , file_size : Int
    , mimetype : String
    , datetime : Int
    , userDate : Maybe Int
    , caption : Maybe String
    , location : Maybe String
    , duration : Maybe Float
    , tags : List String
    }


initialModel : Route -> Model
initialModel route =
    { tagList = RemoteData.NotAsked
    , yearList = RemoteData.NotAsked
    , locationList = RemoteData.NotAsked
    , assetList = RemoteData.NotAsked
    , pageNumber = 1
    , asset = RemoteData.NotAsked
    , route = route
    , assetEditForm = initialAssetEditForm
    , showingAllTags = False
    , showingAllLocations = False
    }


-- Define here so it is easily included elsewhere.
pageSize : Int
pageSize =
    18


initialAssetEditForm : Forms.Form
initialAssetEditForm =
    Forms.initForm assetEditFormFields


-- Fields for the asset edit screen.
assetEditFormFields : List ( String, List Forms.FieldValidator )
assetEditFormFields =
    [ ( "location", [] )
    , ( "caption", [] )
    , ( "tags", [] )
    , ( "user_date", userDateValidations )
    ]


userDateValidations : List Forms.FieldValidator
userDateValidations =
    [ validateUserDate ]


{-| The regular expression describing the format for custom dates.
-}
userDateRegex : String
userDateRegex =
    "^\\d{1,4}-\\d{1,2}-\\d{1,2} \\d{1,2}:\\d{2}$"


{-| Validates the user date field.
-}
validateUserDate : String -> Maybe String
validateUserDate input =
    if String.length input == 0 then
        Nothing
    else if Regex.contains (Regex.regex userDateRegex) input then
        case Date.fromString (String.join "T" (String.split " " input)) of
            Ok value ->
                Nothing
            Err msg ->
                Just msg
    else
        Just "date/time format must be yyyy-mm-dd HH:MM"


{-| Convert UTC milliseconds to our date/time string.
-}
intToDateString : Int -> String
intToDateString num =
    let
        zeroPad len num =
            String.padLeft len '0' (toString num)
        date =
            Date.fromTime (toFloat num)
        month =
            case Date.month date of
                Date.Jan -> "01"
                Date.Feb -> "02"
                Date.Mar -> "03"
                Date.Apr -> "04"
                Date.May -> "05"
                Date.Jun -> "06"
                Date.Jul -> "07"
                Date.Aug -> "08"
                Date.Sep -> "09"
                Date.Oct -> "10"
                Date.Nov -> "11"
                Date.Dec -> "12"
        dateStr =
            String.join "-"
                [ zeroPad 4 (Date.year date)
                , month
                , zeroPad 2 (Date.day date)
                ]
        timeStr =
            String.join ":"
                [ zeroPad 2 (Date.hour date)
                , zeroPad 2 (Date.minute date)
                ]
    in
        String.join " " [dateStr, timeStr]


{-| Convert an optional user date/time value from UTC milliseconds to our
date string (e.g. "2003/05/26 08:30").
-}
userDateToString : Maybe Int -> String
userDateToString userDate =
    case userDate of
        Just num ->
            intToDateString num
        Nothing ->
            ""


{-| Parse the user date/time string into UTC milliseconds. Returns Nothing if
the string cannot be parsed as a date.
-}
userDateStrToInt : String -> Maybe Int
userDateStrToInt userDate =
    let
        dateResult =
            -- convert user input to ISO 8601 which Elm's Date expects
            Date.fromString (String.join "T" (String.split " " userDate))
    in
        case dateResult of
            Ok value ->
                Just (round (Time.inMilliseconds (Date.toTime value)))
            Err msg ->
                Nothing
