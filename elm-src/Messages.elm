module Messages exposing (..)

import Model exposing (..)
import Navigation
import RemoteData exposing (WebData)
import Routing exposing (Route)


type Msg
    = FetchTags (WebData TagList)
    | FetchYears (WebData YearList)
    | FetchLocations (WebData LocationList)
    | ToggleTag String
    | ToggleYear Int
    | ToggleLocation String
    | ToggleAllTags
    | ToggleAllLocations
    | QueryAssets (WebData AssetList)
    | Paginate Int
    | FetchAsset (WebData AssetDetails)
    | UpdateFormAssetEdit String String
    | SubmitAsset String
    | PostSubmitAsset String (WebData AssetDetails)
    | UrlChange Navigation.Location
    | NavigateTo Route
