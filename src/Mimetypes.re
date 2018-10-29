/* Represents the mimetype of an asset. */
type mediaType =
  | Image
  | Video
  | Audio
  | Text
  | Pdf
  | Unknown;

/* Map a filename to one of the MimeType values. */
let filenameToMediaType = (filename: string): mediaType => {
  let parts = Js.String.splitByRe([%bs.re "/\\./"], filename);
  let length = Array.length(parts);
  let ext = length > 0 ? parts[length - 1] : "";
  switch (ext) {
  | "gif" => Image
  | "heic" => Image
  | "heif" => Image
  | "jpeg" => Image
  | "jpg" => Image
  | "png" => Image
  | "tif" => Image
  | "tiff" => Image
  | "aac" => Audio
  | "aif" => Audio
  | "aifc" => Audio
  | "aiff" => Audio
  | "au" => Audio
  | "avi" => Audio
  | "m4a" => Audio
  | "mp4a" => Audio
  | "m4p" => Audio
  | "oga" => Audio
  | "ogg" => Audio
  | "snd" => Audio
  | "weba" => Audio
  | "wma" => Audio
  | "wav" => Audio
  | "mp4" => Video
  | "mp4v" => Video
  | "mpg4" => Video
  | "m4v" => Video
  | "mpeg" => Video
  | "mpg" => Video
  | "ogv" => Video
  | "mov" => Video
  | "qt" => Video
  | "webm" => Video
  | "wmv" => Video
  | "pdf" => Pdf
  | "md" => Text
  | "text" => Text
  | "txt" => Text
  | _ => Unknown
  };
};