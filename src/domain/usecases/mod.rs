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
pub mod ingest;
pub mod location;
pub mod recent;
pub mod search;
pub mod tags;
pub mod types;
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
    } else if media_type.type_() == mime::VIDEO {
        // check for certain types of video formats
        let sub = media_type.subtype().as_str();
        if sub == "x-msvideo" || sub == "vnd.avi" || sub == "avi" || sub == "msvideo" {
            return get_avi_date(filepath);
        }
        // For any other type of video, just hope that it is mp4 compatible.
        fn err_convert(err: mp4::Error) -> Error {
            err_msg(format!("{:?}", err))
        }
        let file = File::open(filepath)?;
        let bmff = mp4::read_mp4(file).map_err(err_convert)?;
        let moov = bmff.moov.ok_or_else(|| err_msg("missing moov atom"))?;
        let creation_time = if moov.mvhd.creation_time > 2082844800 {
            // subtract the difference in seconds between 1904-01-01 and UTC
            // epoch for those times that are clearly not "Unix time"
            moov.mvhd.creation_time - 2082844800
        } else {
            moov.mvhd.creation_time
        };
        return Ok(Utc.timestamp(creation_time as i64, 0));
    }
    Err(err_msg("could not read any date"))
}

// Try reading the date from a RIFF-encoded AVI file.
fn get_avi_date(filepath: &Path) -> Result<DateTime<Utc>, Error> {
    let mut file = File::open(filepath)?;
    let chunk = riff::Chunk::read(&mut file, 0)?;
    if chunk.id() == riff::RIFF_ID {
        let chunk_type = chunk.read_type(&mut file)?;
        if chunk_type.as_str() == "AVI " {
            if let Some(contents) = read_chunk(&chunk, &mut file) {
                if let Some(idit) = find_data("IDIT", &contents) {
                    if let Some(date) = parse_idit_date(&idit) {
                        return Ok(date);
                    }
                }
                // another possible field is DTIM but that requires
                // conversion as noted in the RIFF wikipedia article
                // (https://en.wikipedia.org/wiki/Resource_Interchange_File_Format)
            }
            return Err(err_msg("AVI does not contain a date"));
        }
    }
    Err(err_msg("not RIFF encoded AVI"))
}

//
// Example output for an AVI MJPEG file from 2009. The IDIT field contains a
// date string and the ISFT field contains camera manufacturer/model.
//
// children -> id: 'RIFF', typ: 'AVI ', len: 5
//   children -> id: 'LIST', typ: 'hdrl', len: 5
//     data -> id: 'avih', len: 56
//     children -> id: 'LIST', typ: 'strl', len: 3
//       data -> id: 'strh', len: 56
//       data -> id: 'strf', len: 40
//       data -> id: 'indx', len: 120
//     children -> id: 'LIST', typ: 'strl', len: 3
//       data -> id: 'strh', len: 56
//       data -> id: 'strf', len: 16
//       data -> id: 'indx', len: 120
//     children -> id: 'LIST', typ: 'odml', len: 1
//       data -> id: 'dmlh', len: 248
//     data -> id: 'IDIT', len: 26
//   children -> id: 'LIST', typ: 'INFO', len: 1
//     data -> id: 'ISFT', len: 12
//   data -> id: 'JUNK', len: 1138
//   children -> id: 'LIST', typ: 'movi', len: 2738
//   data -> id: 'idx1', len: 32

fn read_chunk<T>(chunk: &riff::Chunk, file: &mut T) -> Option<riff::ChunkContents>
where
    T: std::io::Seek + std::io::Read,
{
    let id = chunk.id();
    if id == riff::RIFF_ID || id == riff::LIST_ID {
        let chunk_type = chunk.read_type(file).unwrap();
        let children = read_items(&mut chunk.iter(file));
        let mut children_contents: Vec<riff::ChunkContents> = Vec::new();
        for child in children {
            if let Some(contents) = read_chunk(&child, file) {
                children_contents.push(contents);
            }
        }
        Some(riff::ChunkContents::Children(
            id,
            chunk_type,
            children_contents,
        ))
    } else if id == riff::SEQT_ID {
        let children = read_items(&mut chunk.iter_no_type(file));
        let mut children_contents: Vec<riff::ChunkContents> = Vec::new();
        for child in children {
            if let Some(contents) = read_chunk(&child, file) {
                children_contents.push(contents);
            }
        }
        Some(riff::ChunkContents::ChildrenNoType(id, children_contents))
    } else if chunk.len() <= 256 {
        // only interested in the smaller data fields
        let contents = chunk.read_contents(file).unwrap();
        Some(riff::ChunkContents::Data(id, contents))
    } else {
        // ignore everything else, do not allocate memory
        None
    }
}

fn read_items<T>(iter: &mut T) -> Vec<T::Item>
where
    T: Iterator,
{
    let mut vec: Vec<T::Item> = Vec::new();
    for item in iter {
        vec.push(item);
    }
    vec
}

// Scan recursively through the contents to find a named data field.
fn find_data(label: &str, contents: &riff::ChunkContents) -> Option<Vec<u8>> {
    match contents {
        riff::ChunkContents::Data(id, data) => {
            if id.as_str() == label {
                return Some(data.to_owned());
            } else {
                return None;
            }
        }
        riff::ChunkContents::Children(_id, _typ, more) => {
            for content in more.iter() {
                let data = find_data(label, &content);
                if data.is_some() {
                    return data;
                }
            }
        }
        riff::ChunkContents::ChildrenNoType(_id, more) => {
            for content in more.iter() {
                let data = find_data(label, &content);
                if data.is_some() {
                    return data;
                }
            }
        }
    }
    None
}

// Parse the date string found in the IDIT data field.
fn parse_idit_date(bytes: &[u8]) -> Option<DateTime<Utc>> {
    let mut no_nulls = bytes.to_vec();
    no_nulls.retain(|e| *e != 0);
    if let Ok(string) = String::from_utf8(no_nulls) {
        // the date parsing is sensitive to any kind of whitespace
        let value = string.trim();
        // example from a Canon camera: SAT DEC 19 05:46:12 2009
        if let Ok(date) = Utc.datetime_from_str(value, "%a %b %d %H:%M:%S %Y") {
            return Some(date);
        }
        // example from a Samsung camera: 2005:08:17 11:42:43
        if let Ok(date) = Utc.datetime_from_str(value, "%Y:%m:%d %H:%M:%S") {
            return Some(date);
        }
        // example from a Fujifilm camera: Mon Mar  3 09:44:56 2008
        if let Ok(date) = Utc.datetime_from_str(value, "%a %b %e %H:%M:%S %Y") {
            return Some(date);
        }
    }
    None
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

        // not an actual image, despite the media type
        let filename = "./tests/fixtures/lorem-ipsum.txt";
        let filepath = Path::new(filename);
        let actual = get_original_date(&mt, filepath);
        assert!(actual.is_err());

        // MP4-encoded quicktime/mpeg video file
        let filename = "./tests/fixtures/100_1206.MOV";
        let mt: mime::Mime = "video/mp4".parse().unwrap();
        let filepath = Path::new(filename);
        let actual = get_original_date(&mt, filepath);
        assert!(actual.is_ok());
        let date = actual.unwrap();
        assert_eq!(date.year(), 2007);
        assert_eq!(date.month(), 9);
        assert_eq!(date.day(), 14);

        // RIFF-encoded AVI video file
        let filename = "./tests/fixtures/MVI_0727.AVI";
        let mt: mime::Mime = "video/x-msvideo".parse().unwrap();
        let filepath = Path::new(filename);
        let actual = get_original_date(&mt, filepath);
        assert!(actual.is_ok());
        let date = actual.unwrap();
        assert_eq!(date.year(), 2009);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 19);

        // not an actual video, despite the media type
        let filename = "./tests/fixtures/lorem-ipsum.txt";
        let filepath = Path::new(filename);
        let actual = get_original_date(&mt, filepath);
        assert!(actual.is_err());
    }

    #[test]
    fn test_get_avi_date() {
        let filename = "./tests/fixtures/MVI_0727.AVI";
        let filepath = Path::new(filename);
        let result = get_avi_date(filepath);
        assert!(result.is_ok());
        let date = result.unwrap();
        assert_eq!(date.year(), 2009);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 19);
    }

    #[test]
    fn test_parse_idit_date() {
        // example from Canon camera: SAT DEC 19 05:46:12 2009
        let input = vec![
            83, 65, 84, 32, 68, 69, 67, 32, 49, 57, 32, 48, 53, 58, 52, 54, 58, 49, 50, 32, 50, 48,
            48, 57, 10, 0,
        ];
        let option = parse_idit_date(&input);
        assert!(option.is_some());
        let actual = option.unwrap();
        assert_eq!(actual.year(), 2009);
        assert_eq!(actual.month(), 12);
        assert_eq!(actual.day(), 19);
        // example from a Fujifilm camera: Mon Mar  3 09:44:56 2008
        let input = vec![
            77, 111, 110, 32, 77, 97, 114, 32, 32, 51, 32, 48, 57, 58, 52, 52, 58, 53, 54, 32, 50,
            48, 48, 56,
        ];
        let option = parse_idit_date(&input);
        assert!(option.is_some());
        let actual = option.unwrap();
        assert_eq!(actual.year(), 2008);
        assert_eq!(actual.month(), 3);
        assert_eq!(actual.day(), 3);
        // example from a Samsung camera: 2005:08:17 11:42:43
        let input = vec![
            50, 48, 48, 53, 58, 48, 56, 58, 49, 55, 32, 49, 49, 58, 52, 50, 58, 52, 51,
        ];
        let option = parse_idit_date(&input);
        assert!(option.is_some());
        let actual = option.unwrap();
        assert_eq!(actual.year(), 2005);
        assert_eq!(actual.month(), 8);
        assert_eq!(actual.day(), 17);
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
