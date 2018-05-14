module Messages exposing (..)

import Model exposing (..)
import Navigation
import Routing exposing (Route)


type Msg
    = TagsResponse (GraphResult TagList)
    | YearsResponse (GraphResult YearList)
    | LocationsResponse (GraphResult LocationList)
    | ToggleTag String
    | ToggleYear Int
    | ToggleLocation String
    | ToggleAllTags
    | ToggleAllLocations
    | AssetsResponse (GraphResult AssetList)
    | ThumblessAsset String
    | Paginate Int
    | AssetResponse (GraphResult AssetDetails)
    | UpdateFormAssetEdit String String
    | SubmitAsset String
    | UploadSelection String
    | SubmitResponse String (GraphResult AssetDetails)
    | SearchAssets
    | UpdateFormSearch String String
    | UrlChange Navigation.Location
    | NavigateTo Route
