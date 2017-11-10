module Routing exposing (..)

import Navigation
import UrlParser exposing (..)


type Route
    = HomeIndexRoute
    | NotFoundRoute
    | ShowAssetRoute String
    | EditAssetRoute String


matchers : Parser (Route -> a) a
matchers =
    oneOf
        [ map HomeIndexRoute <| s ""
        , map ShowAssetRoute <| s "assets" </> string
        , map EditAssetRoute <| s "assets" </> string </> s "edit"
        ]


parse : Navigation.Location -> Route
parse location =
    case UrlParser.parsePath matchers location of
        Just route ->
            route

        Nothing ->
            NotFoundRoute


toPath : Route -> String
toPath route =
    case route of
        HomeIndexRoute ->
            "/"

        NotFoundRoute ->
            "/not-found"

        ShowAssetRoute id ->
            "/assets/" ++ id

        EditAssetRoute id ->
            "/assets/" ++ id ++ "/edit"
