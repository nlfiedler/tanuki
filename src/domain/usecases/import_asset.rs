//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::domain::entities::Asset;
use crate::domain::repositories::BlobRepository;
use crate::domain::repositories::RecordRepository;
use chrono::prelude::*;
use failure::Error;
use rusty_ulid::generate_ulid_string;
use std::cmp;
use std::ffi::OsStr;
use std::fmt;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

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
        let asset_id = new_asset_id(now, &params.filepath);
        let filename = get_file_name(&params.filepath);
        let metadata = std::fs::metadata(&params.filepath)?;
        let byte_length = metadata.len();
        let media_type = detect_media_type(&filename);
        let asset = Asset {
            key: asset_id,
            checksum: digest,
            filename,
            byte_length,
            media_type: media_type.to_string(),
            tags: vec![],
            import_date: now,
            location: None,
            duration: None,
            user_date: None,
            original_date: None,
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
}

impl Params {
    pub fn new(filepath: PathBuf) -> Self {
        Self { filepath }
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
/// The identifier is suitable to be used as a file path within blob storage.
///
/// This is _not_ a pure function, since it involves both the current time as
/// well as a random number. It does, however, avoid any possibility of name
/// collisions.
///
fn new_asset_id(datetime: DateTime<Utc>, filename: &Path) -> String {
    // round the date/time down to the nearest quarter hour
    // (e.g. 21:50 becomes 21:45, 08:10 becomes 08:00)
    let minutes = (datetime.minute() / 15) * 15;
    let round_date = datetime.with_minute(minutes).unwrap();
    let mut leading_path = round_date.format("%Y/%m/%d/%H%M/").to_string();
    let extension = filename.extension().map(OsStr::to_str).flatten();
    let mut name = generate_ulid_string();
    if let Some(ext) = extension {
        name.push('.');
        name.push_str(ext);
    }
    leading_path.push_str(&name);
    let rel_path = leading_path.to_lowercase();
    base64::encode(&rel_path)
}

///
/// Return the first guessed media type based on the file name.
///
fn detect_media_type(filename: &str) -> mime::Mime {
    // Alternatively could use a crate that reads the content and guesses at the
    // media type (e.g. https://github.com/flier/rust-mime-sniffer), perhaps as
    // a fallback when the filename-based guess yields "octet-stream".
    let guess = mime_guess::from_path(filename);
    guess.first_or_octet_stream()
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
        let actual = new_asset_id(import_date, Path::new(filename));
        // Cannot compare the actual value because it incorporates the current
        // time and a random number, can only decode and check the basic format
        // matches expectations.
        let decoded = base64::decode(&actual).unwrap();
        let as_string = String::from_utf8(decoded).unwrap();
        assert!(as_string.starts_with("2018/05/31/2100/"));
        assert!(as_string.ends_with(".jpg"));
        assert_eq!(as_string.len(), 46);
    }

    #[test]
    fn test_get_file_name() {
        let filepath = Path::new("./test/fixtures/fighting_kittens.jpg");
        let actual = get_file_name(&filepath);
        assert_eq!(actual, "fighting_kittens.jpg");
    }

    #[test]
    fn test_detect_media_type() {
        assert_eq!(detect_media_type("image.jpg"), mime::IMAGE_JPEG);
        let video_quick: mime::Mime = "video/quicktime".parse().unwrap();
        assert_eq!(detect_media_type("video.mov"), video_quick);
        let video_mpeg: mime::Mime = "video/mpeg".parse().unwrap();
        assert_eq!(detect_media_type("video.mpg"), video_mpeg);
        assert_eq!(
            // does not yet guess the apple image type
            detect_media_type("image.heic"),
            mime::APPLICATION_OCTET_STREAM
        );
    }

    #[test]
    fn test_import_asset_new() {
        // arrange
        let digest = "sha256-82084759e4c766e94bb91d8cf9ed9edc1d4480025205f5109ec39a806509ee09";
        let infile = PathBuf::from("./test/fixtures/fighting_kittens.jpg");
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
        let params = Params::new(infile);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let asset = result.unwrap();
        assert_eq!(asset.checksum, digest);
        assert_eq!(asset.filename, "fighting_kittens.jpg");
        assert_eq!(asset.media_type, "image/jpeg");
        assert!(asset.tags.is_empty());
    }

    #[test]
    fn test_import_asset_existing() {
        // arrange
        let digest = "sha256-82084759e4c766e94bb91d8cf9ed9edc1d4480025205f5109ec39a806509ee09";
        let asset1 = Asset {
            key: "abc123".to_owned(),
            checksum: digest.to_owned(),
            filename: "fighting_kittens.jpg".to_owned(),
            byte_length: 39932,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kittens".to_owned()],
            import_date: Utc::now(),
            location: None,
            duration: None,
            user_date: None,
            original_date: None,
        };
        let mut records = MockRecordRepository::new();
        records
            .expect_get_asset_by_digest()
            .with(eq(digest))
            .returning(move |_| Ok(Some(asset1.clone())));
        let blobs = MockBlobRepository::new();
        // act
        let usecase = ImportAsset::new(Box::new(records), Box::new(blobs));
        let infile = PathBuf::from("./test/fixtures/fighting_kittens.jpg");
        let params = Params::new(infile);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let asset = result.unwrap();
        assert_eq!(asset.checksum, digest);
        assert_eq!(asset.filename, "fighting_kittens.jpg");
    }
}