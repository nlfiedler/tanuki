module Mimetypes exposing (..)


{- Represents the mimetype of an asset.
-}
type MimeType
    = Image
    | Video
    | Audio
    | Text
    | Pdf
    | Unknown


{- Map a filename to one of the MimeType values.
-}
filenameToMimetype : String -> MimeType
filenameToMimetype filename =
    let
        ext =
            filename
            |> String.split "."
            |> List.reverse
            |> List.head
            |> Maybe.withDefault ""
    in
        -- Would like to define a static map, but Elm does not have that yet;
        -- hopefully this compiles to something efficient anyway.
        case ext of
            "gif" ->
                Image
            "heic" ->
                Image
            "heif" ->
                Image
            "jpeg" ->
                Image
            "jpg" ->
                Image
            "png" ->
                Image
            "tif" ->
                Image
            "tiff" ->
                Image
            "aac" ->
                Audio
            "aif" ->
                Audio
            "aifc" ->
                Audio
            "aiff" ->
                Audio
            "au" ->
                Audio
            "avi" ->
                Audio
            "m4a" ->
                Audio
            "mp4a" ->
                Audio
            "m4p" ->
                Audio
            "oga" ->
                Audio
            "ogg" ->
                Audio
            "snd" ->
                Audio
            "weba" ->
                Audio
            "wma" ->
                Audio
            "wav" ->
                Audio
            "mp4" ->
                Video
            "mp4v" ->
                Video
            "mpg4" ->
                Video
            "m4v" ->
                Video
            "mpeg" ->
                Video
            "mpg" ->
                Video
            "ogv" ->
                Video
            "mov" ->
                Video
            "qt" ->
                Video
            "webm" ->
                Video
            "wmv" ->
                Video
            "pdf" ->
                Pdf
            "text" ->
                Text
            "txt" ->
                Text
            _ ->
                Unknown
