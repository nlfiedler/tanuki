module Model exposing (..)

import Date exposing (Month(..))
import Date.Extra as Date
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
    , assetSearchForm : Forms.Form
    , savedSearch : SearchValues
    , showingAllTags : Bool
    , showingAllLocations : Bool
    , hasDragSupport : Bool
    , uploadFilename : Maybe String
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


-- Saved search form field values.
type alias SearchValues =
    { tags : String
    , locations : String
    , before : String
    , after : String
    , filename : String
    , mimetype : String
    }


{-| Create the initial model. The draggable flag indicates drag&drop support,
as determined using native JavaScript.
-}
initialModel : Route -> Bool -> Model
initialModel route draggable =
    { tagList = RemoteData.NotAsked
    , yearList = RemoteData.NotAsked
    , locationList = RemoteData.NotAsked
    , assetList = RemoteData.NotAsked
    , pageNumber = 1
    , asset = RemoteData.NotAsked
    , route = route
    , assetEditForm = initialAssetEditForm
    , assetSearchForm = initialSearchForm
    , savedSearch = initialSearchValues
    , showingAllTags = False
    , showingAllLocations = False
    , hasDragSupport = draggable
    , uploadFilename = Nothing
    }


-- Define here so it is easily included elsewhere.
pageSize : Int
pageSize =
    18


initialSearchForm : Forms.Form
initialSearchForm =
    Forms.initForm searchFormFields


-- Fields for the asset edit screen.
searchFormFields : List ( String, List Forms.FieldValidator )
searchFormFields =
    [ ( "tags", [] )
    , ( "locations", [] )
    , ( "before", rangeDateValidations )
    , ( "after", rangeDateValidations )
    , ( "filename", [] )
    , ( "mimetype", [] )
    ]


rangeDateValidations : List Forms.FieldValidator
rangeDateValidations =
    [ validateRangeDate ]


{-| Validates the "range" date field, which has yyyy-MM-dd only (no time).
-}
validateRangeDate : String -> Maybe String
validateRangeDate input =
    if String.length input == 0 then
        Nothing
    else if Regex.contains (Regex.regex "^\\d{1,4}-\\d{1,2}-\\d{1,2}$") input then
        case (Date.fromIsoString input) of
            Ok value ->
                Nothing
            Err msg ->
                Just msg
    else
        Just "date format must be yyyy-MM-dd"


{-| Parse the range date string into UTC milliseconds. Returns Nothing if
the string cannot be parsed as a date.
-}
rangeDateStrToInt : String -> Maybe Int
rangeDateStrToInt rangeDate =
    let
        dateResult =
            -- Convert user input to ISO 8601 format, without a trailing Z, so
            -- that the date will be treated as local time. Date.Extra
            -- represents the time as UTC plus an offset, so using Date.toTime
            -- will return the UTC time.
            Date.fromIsoString rangeDate
    in
        case dateResult of
            Ok value ->
                Just (round (Time.inMilliseconds (Date.toTime value)))
            Err msg ->
                Nothing


initialSearchValues : SearchValues
initialSearchValues =
    { tags = ""
    , locations = ""
    , before = ""
    , after = ""
    , filename = ""
    , mimetype = ""
    }


initialAssetEditForm : Forms.Form
initialAssetEditForm =
    Forms.initForm assetEditFormFields


-- Fields for the asset edit screen.
assetEditFormFields : List ( String, List Forms.FieldValidator )
assetEditFormFields =
    [ ( "location", [] )
    , ( "caption", [] )
    , ( "mimetype", [] )
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
        case Date.fromIsoString (String.join "T" (String.split " " input)) of
            Ok value ->
                Nothing
            Err msg ->
                Just msg
    else
        Just "date/time format must be yyyy-MM-dd HH:mm"

{-| Create a Date for the given year, as January 1st at midnight.
Returns the UTC milliseconds.
-}
dateForYear : Int -> Maybe Int
dateForYear year =
    let
        value =
            Date.fromParts year Jan 1 0 0 0 0
    in
        Just (round (Time.inMilliseconds (Date.toTime value)))


{-| Convert UTC milliseconds to our date/time string.
-}
intToDateString : Int -> String
intToDateString num =
    -- toFormattedString uses local time
    Date.toFormattedString
        "yyyy-MM-dd HH:mm"
        (Date.fromTime (toFloat num))


{-| Convert an optional user date/time value from UTC milliseconds to our
date string (e.g. "2003-05-26 08:30").
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
            -- Convert user input to ISO 8601 format, without a trailing Z, so
            -- that the date will be treated as local time. Date.Extra
            -- represents the time as UTC plus an offset, so using Date.toTime
            -- will return the UTC time.
            Date.fromIsoString (String.join "T" (String.split " " userDate))
    in
        case dateResult of
            Ok value ->
                Just (round (Time.inMilliseconds (Date.toTime value)))
            Err msg ->
                Nothing


{- Get the selected tags from the model.

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


{- Get the selected years from the model.

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


{- Get the selected locations from the model.

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
