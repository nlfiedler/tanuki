module Main exposing (..)

import Model exposing (..)
import Messages exposing (..)
import Navigation
import Routing exposing (parse)
import Update exposing (..)
import View exposing (view)


-- Flags passed from the JavaScript code that invokes our main.
type alias Flags =
    { draggable : Bool
    }


init : Flags -> Navigation.Location -> ( Model, Cmd Msg )
init flags location =
    let
        currentRoute =
            parse location
        model =
            initialModel currentRoute flags.draggable
    in
        urlUpdate model


main : Program Flags Model Msg
main =
    Navigation.programWithFlags UrlChange
        { init = init
        , view = view
        , update = update
        , subscriptions = always <| Sub.none
        }
