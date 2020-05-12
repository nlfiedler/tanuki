//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::domain::entities::{Asset, Dimensions};
use crate::domain::repositories::BlobRepository;
use crate::domain::repositories::RecordRepository;
use chrono::prelude::*;
use failure::{err_msg, Error};
use rusty_ulid::generate_ulid_string;
use std::cmp;
use std::ffi::OsStr;
use std::fmt;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::str;

pub struct ImportAsset {
    records: Box<dyn RecordRepository>,
    blobs: Box<dyn BlobRepository>,
}

impl ImportAsset {
    pub fn new(records: Box<dyn RecordRepository>, blobs: Box<dyn BlobRepository>) -> Self {
        Self { records, blobs }
    }

    // Create an asset entity based on available information.
    fn create_asset(&self, digest: String, params: Params) -> Result<Asset, Error> {
        let now = Utc::now();
        let asset_id = new_asset_id(now, &params.filepath, &params.media_type);
        let filename = get_file_name(&params.filepath);
        let metadata = std::fs::metadata(&params.filepath)?;
        let byte_length = metadata.len();
        let original_date = get_original_date(&params.media_type, &params.filepath).ok();
        let dimensions = get_dimensions(&params.media_type, &params.filepath).ok();
        let asset = Asset {
            key: asset_id,
            checksum: digest,
            filename,
            byte_length,
            media_type: params.media_type.to_string(),
            tags: vec![],
            import_date: now,
            caption: None,
            location: None,
            user_date: None,
            original_date,
            dimensions,
        };
        Ok(asset)
    }
}

impl super::UseCase<Asset, Params> for ImportAsset {
    fn call(&self, params: Params) -> Result<Asset, Error> {
        let digest = checksum_file(&params.filepath)?;
        let asset = match self.records.get_asset_by_digest(&digest)? {
            Some(a) => a,
            None => {
                let asset = self.create_asset(digest, params.clone())?;
                self.records.put_asset(&asset)?;
                self.blobs.store_blob(&params.filepath, &asset)?;
                asset
            }
        };
        Ok(asset)
    }
}

#[derive(Clone)]
pub struct Params {
    filepath: PathBuf,
    media_type: mime::Mime,
}

impl Params {
    pub fn new(filepath: PathBuf, media_type: mime::Mime) -> Self {
        Self {
            filepath,
            media_type,
        }
    }
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Params({:?})", self.filepath)
    }
}

impl cmp::PartialEq for Params {
    fn eq(&self, other: &Self) -> bool {
        self.filepath == other.filepath
    }
}

impl cmp::Eq for Params {}

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
/// Return the last part of the path, converting to a String.
///
fn get_file_name(path: &Path) -> String {
    // ignore any paths that end in '..'
    if let Some(p) = path.file_name() {
        // ignore any paths that failed UTF-8 translation
        if let Some(pp) = p.to_str() {
            return pp.to_owned();
        }
    }
    // normal conversion failed, return whatever garbage is there
    path.to_string_lossy().into_owned()
}

///
/// Use the datetime and filename to produce a relative path, and return as a
/// base64 encoded value, suitable as an identifier.
///
/// The decoded identifier is suitable to be used as a file path within blob
/// storage. The filename will be universally unique and the path will ensure
/// assets are distributed across directories to avoid congestion.
///
/// This is _not_ a pure function, since it involves a random number. It does,
/// however, avoid any possibility of name collisions.
///
fn new_asset_id(datetime: DateTime<Utc>, filepath: &Path, media_type: &mime::Mime) -> String {
    // Round the date/time down to the nearest quarter hour (e.g. 21:50 becomes
    // 21:45, 08:10 becomes 08:00) to avoid creating many directories with only
    // a few assets in them.
    let minutes = (datetime.minute() / 15) * 15;
    let round_date = datetime.with_minute(minutes).unwrap();
    let mut leading_path = round_date.format("%Y/%m/%d/%H%M/").to_string();
    let extension = filepath.extension().map(OsStr::to_str).flatten();
    let mut name = generate_ulid_string();
    let append_suffix = if let Some(ext) = extension {
        name.push('.');
        name.push_str(ext);
        let guessed_mime = detect_media_type(ext);
        &guessed_mime != media_type
    } else {
        true
    };
    if append_suffix {
        // If the media type guessed from the file extension differs from the
        // media type provided in the parameters, then append the preferred
        // suffix to the name. This provides a means for the blob repository to
        // correctly guess the media type from just the asset identifier.
        let maybe_mime_extension = mime_guess::get_mime_extensions(media_type).map(|l| l[0]);
        if let Some(mime_ext) = maybe_mime_extension {
            name.push('.');
            name.push_str(mime_ext);
        }
    }
    leading_path.push_str(&name);
    let rel_path = leading_path.to_lowercase();
    base64::encode(&rel_path)
}

//
// Return the first guessed media type based on the extension.
//
fn detect_media_type(extension: &str) -> mime::Mime {
    // Alternatively could use a crate that reads the content and guesses at the
    // media type (e.g. https://github.com/flier/rust-mime-sniffer), perhaps as
    // a fallback when the extension-based guess yields "octet-stream".
    let guess = mime_guess::from_ext(extension);
    guess.first_or_octet_stream()
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

///
/// Gather the pixel dimensions of the image asset.
///
/// Returns an error if unsuccessful.
///
fn get_dimensions(media_type: &mime::Mime, filepath: &Path) -> Result<Dimensions, Error> {
    if media_type.type_() == mime::IMAGE {
        let dim = image::image_dimensions(filepath)?;
        return Ok(Dimensions(dim.0, dim.1));
    }
    Err(err_msg("not an image"))
}

#[cfg(test)]
mod tests {
    use super::super::UseCase;
    use super::*;
    use crate::domain::repositories::MockBlobRepository;
    use crate::domain::repositories::MockRecordRepository;
    use mockall::predicate::*;

    #[test]
    fn test_checksum_file() -> Result<(), io::Error> {
        let infile = Path::new("./test/fixtures/fighting_kittens.jpg");
        let sha256 = checksum_file(&infile)?;
        assert_eq!(
            sha256,
            "sha256-82084759e4c766e94bb91d8cf9ed9edc1d4480025205f5109ec39a806509ee09"
        );
        Ok(())
    }

    #[test]
    fn test_new_asset_id() {
        let import_date = Utc.ymd(2018, 5, 31).and_hms(21, 10, 11);
        let filename = "fighting_kittens.jpg";
        let mt = mime::IMAGE_JPEG;
        let actual = new_asset_id(import_date, Path::new(filename), &mt);
        // Cannot compare the actual value because it incorporates a random
        // number, can only decode and check the basic format matches
        // expectations.
        let decoded = base64::decode(&actual).unwrap();
        let as_string = std::str::from_utf8(&decoded).unwrap();
        assert!(as_string.starts_with("2018/05/31/2100/"));
        assert!(as_string.ends_with(".jpg"));
        assert_eq!(as_string.len(), 46);

        // test with an image/jpeg asset with an incorrect extension
        let filename = "fighting_kittens.foo";
        let actual = new_asset_id(import_date, Path::new(filename), &mt);
        let decoded = base64::decode(&actual).unwrap();
        let as_string = std::str::from_utf8(&decoded).unwrap();
        // jpe because it's first in the list of [jpe, jpeg, jpg]
        assert!(as_string.ends_with(".foo.jpe"));

        // test with an image/jpeg asset with _no_ extension
        let filename = "fighting_kittens";
        let actual = new_asset_id(import_date, Path::new(filename), &mt);
        let decoded = base64::decode(&actual).unwrap();
        let as_string = std::str::from_utf8(&decoded).unwrap();
        // jpe because it's first in the list of [jpe, jpeg, jpg]
        assert!(as_string.ends_with(".jpe"));
    }

    #[test]
    fn test_get_file_name() {
        let filepath = Path::new("./test/fixtures/fighting_kittens.jpg");
        let actual = get_file_name(&filepath);
        assert_eq!(actual, "fighting_kittens.jpg");
    }

    #[test]
    fn test_detect_media_type() {
        assert_eq!(detect_media_type("jpg"), mime::IMAGE_JPEG);
        let video_quick: mime::Mime = "video/quicktime".parse().unwrap();
        assert_eq!(detect_media_type("mov"), video_quick);
        let video_mpeg: mime::Mime = "video/mpeg".parse().unwrap();
        assert_eq!(detect_media_type("mpg"), video_mpeg);
        assert_eq!(
            // does not yet guess the apple image type
            detect_media_type("heic"),
            mime::APPLICATION_OCTET_STREAM
        );
    }

    #[test]
    fn test_import_asset_new() {
        // arrange
        let digest = "sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07";
        let infile = PathBuf::from("./test/fixtures/dcp_1069.jpg");
        let infile_copy = infile.clone();
        let mut records = MockRecordRepository::new();
        records
            .expect_get_asset_by_digest()
            .with(eq(digest))
            .returning(|_| Ok(None));
        records.expect_put_asset().returning(|_| Ok(()));
        let mut blobs = MockBlobRepository::new();
        blobs
            .expect_store_blob()
            .with(function(move |p| p == infile_copy.as_path()), always())
            .returning(|_, _| Ok(()));
        // act
        let usecase = ImportAsset::new(Box::new(records), Box::new(blobs));
        let media_type = mime::IMAGE_JPEG;
        let params = Params::new(infile, media_type);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let asset = result.unwrap();
        assert_eq!(asset.checksum, digest);
        assert_eq!(asset.filename, "dcp_1069.jpg");
        assert_eq!(asset.byte_length, 80977);
        assert_eq!(asset.media_type, "image/jpeg");
        assert!(asset.tags.is_empty());
        assert!(asset.original_date.is_some(), "expected an original date");
        assert_eq!(asset.original_date.unwrap().year(), 2003);
        assert!(asset.dimensions.is_some(), "expected image dimensions");
        assert_eq!(asset.dimensions.as_ref().unwrap().0, 440);
        assert_eq!(asset.dimensions.as_ref().unwrap().1, 292);
    }

    #[test]
    fn test_import_asset_existing() {
        // arrange
        let digest = "sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07";
        let asset1 = Asset {
            key: "abc123".to_owned(),
            checksum: digest.to_owned(),
            filename: "dcp_1069.jpg".to_owned(),
            byte_length: 80977,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cow".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let mut records = MockRecordRepository::new();
        records
            .expect_get_asset_by_digest()
            .with(eq(digest))
            .returning(move |_| Ok(Some(asset1.clone())));
        let blobs = MockBlobRepository::new();
        // act
        let usecase = ImportAsset::new(Box::new(records), Box::new(blobs));
        let infile = PathBuf::from("./test/fixtures/dcp_1069.jpg");
        let media_type = mime::IMAGE_JPEG;
        let params = Params::new(infile, media_type);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let asset = result.unwrap();
        assert_eq!(asset.checksum, digest);
        assert_eq!(asset.filename, "dcp_1069.jpg");
    }

    #[test]
    fn test_get_original_date() {
        // has EXIF header and the original date/time value
        let filename = "./test/fixtures/dcp_1069.jpg";
        let mt = mime::IMAGE_JPEG;
        let filepath = Path::new(filename);
        let actual = get_original_date(&mt, filepath);
        assert!(actual.is_ok());
        let date = actual.unwrap();
        assert_eq!(date.year(), 2003);
        assert_eq!(date.month(), 9);
        assert_eq!(date.day(), 3);

        // does not have date/time original value
        let filename = "./test/fixtures/fighting_kittens.jpg";
        let filepath = Path::new(filename);
        let actual = get_original_date(&mt, filepath);
        assert!(actual.is_err());

        // does not have EXIF header at all
        let filename = "./test/fixtures/animal-cat-cute-126407.jpg";
        let filepath = Path::new(filename);
        let actual = get_original_date(&mt, filepath);
        assert!(actual.is_err());

        // not an image
        let filename = "./test/fixtures/lorem-ipsum.txt";
        let filepath = Path::new(filename);
        let actual = get_original_date(&mt, filepath);
        assert!(actual.is_err());

        // video file
        // let filename = "./test/fixtures/100_1206.MOV";
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
    fn test_get_dimensions() {
        let filename = "./test/fixtures/dcp_1069.jpg";
        let mt = mime::IMAGE_JPEG;
        let filepath = Path::new(filename);
        let actual = get_dimensions(&mt, filepath);
        assert!(actual.is_ok());
        let dim = actual.unwrap();
        assert_eq!(dim.0, 440);
        assert_eq!(dim.1, 292);

        // rotated sideways (dimensions are flipped)
        let filename = "./test/fixtures/fighting_kittens.jpg";
        let filepath = Path::new(filename);
        let actual = get_dimensions(&mt, filepath);
        assert!(actual.is_ok());
        let dim = actual.unwrap();
        assert_eq!(dim.0, 384);
        assert_eq!(dim.1, 512);

        let filename = "./test/fixtures/animal-cat-cute-126407.jpg";
        let filepath = Path::new(filename);
        let actual = get_dimensions(&mt, filepath);
        assert!(actual.is_ok());
        let dim = actual.unwrap();
        assert_eq!(dim.0, 2067);
        assert_eq!(dim.1, 1163);

        // not an image
        let filename = "./test/fixtures/lorem-ipsum.txt";
        let filepath = Path::new(filename);
        let actual = get_dimensions(&mt, filepath);
        assert!(actual.is_err());
    }
}
