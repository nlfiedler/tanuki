module Model exposing (..)

import Forms
import GraphQL.Client.Http as GraphQLClient
import Regex
import RemoteData
import Routing exposing (Route)


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
    , date : String
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
    , datetime : String
    , userDate : Maybe String
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
