//
// Copyright (c) 2020 Nathan Fiedler
//
use chrono::prelude::*;
use failure::{err_msg, Error};
use std::cmp;
use std::fmt;
use std::fs::File;
use std::io;
use std::path::Path;
use std::str;

pub mod checksum;
pub mod count;
pub mod diagnose;
pub mod fetch;
pub mod import;
pub mod location;
pub mod recent;
pub mod search;
pub mod tags;
pub mod update;
pub mod year;

/// `UseCase` is the interface by which all use cases are invoked.
pub trait UseCase<Type, Params> {
    fn call(&self, params: Params) -> Result<Type, Error>;
}

/// `NoParams` is the type for use cases that do not take arguments.
pub struct NoParams {}

impl fmt::Display for NoParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NoParams()")
    }
}

impl cmp::PartialEq for NoParams {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl cmp::Eq for NoParams {}

///
/// Compute the SHA256 hash digest of the given file.
///
fn checksum_file(infile: &Path) -> io::Result<String> {
    use sha2::{Digest, Sha256};
    let mut file = File::open(infile)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    let digest = hasher.result();
    Ok(format!("sha256-{:x}", digest))
}

///
/// Extract the original date/time from the asset.
///
/// Returns an error if unsuccessful.
///
fn get_original_date(media_type: &mime::Mime, filepath: &Path) -> Result<DateTime<Utc>, Error> {
    if media_type.type_() == mime::IMAGE {
        // For now just hope that the image has an EXIF header, while someday
        // can add support for other image formats.
        let file = File::open(filepath)?;
        let mut buffer = io::BufReader::new(&file);
        let reader = exif::Reader::new();
        let exif = reader.read_from_container(&mut buffer)?;
        let field = exif
            .get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY)
            .ok_or_else(|| err_msg("no date/time field"))?;
        if let exif::Value::Ascii(data) = &field.value {
            let value = str::from_utf8(&data[0])?;
            return Utc
                .datetime_from_str(value, "%Y:%m:%d %H:%M:%S")
                .map_err(|_| err_msg("could not parse data"));
        }
        // } else if media_type.type_() == mime::VIDEO {
        //     // For now just hope that the video is mp4 compatible, while someday can
        //     // add support for other video formats.
        //     fn err_convert(err: mp4::Error) -> Error {
        //         err_msg(format!("{:?}", err))
        //     }
        //     let file = File::open(filepath)?;
        //     let bmff = mp4::read_mp4(file).map_err(err_convert)?;
        //     let moov = bmff.moov.ok_or_else(|| err_msg("missing moov atom"))?;
        //     println!("creation_time: {:?}", moov.mvhd.creation_time);
    }
    Err(err_msg("not an image"))
}

//
// Return the first guessed media type based on the extension.
//
fn infer_media_type(extension: &str) -> mime::Mime {
    // Alternatively could use a crate that reads the content and guesses at the
    // media type (e.g. https://github.com/flier/rust-mime-sniffer), perhaps as
    // a fallback when the extension-based guess yields "octet-stream".
    let guess = mime_guess::from_ext(extension);
    guess.first_or_octet_stream()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noparams_equality() {
        let np1 = NoParams {};
        let np2 = NoParams {};
        assert!(np1 == np2);
        assert!(np2 == np1);
    }

    #[test]
    fn test_noparams_stringify() {
        let np = NoParams {};
        assert_eq!(np.to_string(), "NoParams()");
    }

    #[test]
    fn test_checksum_file() -> Result<(), io::Error> {
        let infile = Path::new("./tests/fixtures/fighting_kittens.jpg");
        let sha256 = checksum_file(&infile)?;
        assert_eq!(
            sha256,
            "sha256-82084759e4c766e94bb91d8cf9ed9edc1d4480025205f5109ec39a806509ee09"
        );
        Ok(())
    }

    #[test]
    fn test_get_original_date() {
        // has EXIF header and the original date/time value
        let filename = "./tests/fixtures/dcp_1069.jpg";
        let mt = mime::IMAGE_JPEG;
        let filepath = Path::new(filename);
        let actual = get_original_date(&mt, filepath);
        assert!(actual.is_ok());
        let date = actual.unwrap();
        assert_eq!(date.year(), 2003);
        assert_eq!(date.month(), 9);
        assert_eq!(date.day(), 3);

        // does not have date/time original value
        let filename = "./tests/fixtures/fighting_kittens.jpg";
        let filepath = Path::new(filename);
        let actual = get_original_date(&mt, filepath);
        assert!(actual.is_err());

        // does not have EXIF header at all
        let filename = "./tests/fixtures/animal-cat-cute-126407.jpg";
        let filepath = Path::new(filename);
        let actual = get_original_date(&mt, filepath);
        assert!(actual.is_err());

        // not an image
        let filename = "./tests/fixtures/lorem-ipsum.txt";
        let filepath = Path::new(filename);
        let actual = get_original_date(&mt, filepath);
        assert!(actual.is_err());

        // video file
        // let filename = "./tests/fixtures/100_1206.MOV";
        // let mt: mime::Mime = "video/mp4".parse().unwrap();
        // let filepath = Path::new(filename);
        // let actual = get_original_date(&mt, filepath);
        // assert!(actual.is_ok());
        // let date = actual.unwrap();
        // assert_eq!(date.year(), 2007);
        // assert_eq!(date.month(), 9);
        // assert_eq!(date.day(), 14);
    }

    #[test]
    fn test_infer_media_type() {
        assert_eq!(infer_media_type("jpg"), mime::IMAGE_JPEG);
        let video_quick: mime::Mime = "video/quicktime".parse().unwrap();
        assert_eq!(infer_media_type("mov"), video_quick);
        let video_mpeg: mime::Mime = "video/mpeg".parse().unwrap();
        assert_eq!(infer_media_type("mpg"), video_mpeg);
        assert_eq!(
            // does not yet guess the apple image type
            infer_media_type("heic"),
            mime::APPLICATION_OCTET_STREAM
        );
    }
}
