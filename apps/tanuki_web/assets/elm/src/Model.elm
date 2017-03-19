module Model exposing (..)

import Forms
import Regex
import RemoteData exposing (WebData)
import Routing exposing (Route)


type alias Model =
    { tagList : WebData TagList
    , yearList : WebData YearList
    , locationList : WebData LocationList
    , assetList : WebData AssetList
    , pageNumber : Int
    , asset : WebData AssetDetails
    , route : Route
    , assetEditForm : Forms.Form
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
    , filename : String
    , date : String
    , checksum : String
    }


-- Detailed information on a single asset.
type alias AssetDetails =
    { id : String
    , filename : String
    , size : Int
    , mimetype : String
    , datetime : String
    , userDate : Maybe String
    , checksum : String
    , caption : Maybe String
    , location : Maybe String
    , duration : Maybe Int
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


-- Validates the user date field.
validateUserDate : String -> Maybe String
validateUserDate input =
    let
        -- allow either empty string or yyyy-mm-dd
        dateRegex =
            "^$|^\\d{1,4}-\\d{1,2}-\\d{1,2}$"
    in
        if Regex.contains (Regex.regex dateRegex) input then
            Nothing
        else
            Just "date format must be yyyy-mm-dd"


extractUserDate : AssetDetails -> String
extractUserDate asset =
    let
        userDate =
            case asset.userDate of
                Just date ->
                    -- strip off the artificial time value from the backend
                    Maybe.withDefault "" (List.head (String.split " " date))
                Nothing ->
                    ""
    in
        userDate
