//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::domain::entities::{Asset, Dimensions};
use crate::domain::repositories::BlobRepository;
use crate::domain::repositories::RecordRepository;
use crate::domain::usecases::{checksum_file, get_original_date, infer_media_type};
use chrono::prelude::*;
use anyhow::{anyhow, Error};
use base64::{Engine as _, engine::general_purpose};
use rusty_ulid::generate_ulid_string;
use std::cmp;
use std::ffi::OsStr;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct ImportAsset {
    records: Arc<dyn RecordRepository>,
    blobs: Arc<dyn BlobRepository>,
}

impl ImportAsset {
    pub fn new(records: Arc<dyn RecordRepository>, blobs: Arc<dyn BlobRepository>) -> Self {
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
            Some(asset) => asset,
            None => {
                let asset = self.create_asset(digest, params.clone())?;
                self.records.put_asset(&asset)?;
                asset
            }
        };
        // blob repo will ensure the temporary file is (re)moved
        self.blobs.store_blob(&params.filepath, &asset)?;
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
        let guessed_mime = infer_media_type(ext);
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
    general_purpose::STANDARD.encode(&rel_path)
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
    Err(anyhow!("not an image"))
}

#[cfg(test)]
mod tests {
    use super::super::UseCase;
    use super::*;
    use crate::domain::repositories::MockBlobRepository;
    use crate::domain::repositories::MockRecordRepository;
    use mockall::predicate::*;

    fn make_date_time(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, hour, minute, second)
            .single()
            .unwrap()
    }

    #[test]
    fn test_new_asset_id() {
        let import_date = make_date_time(2018, 5, 31, 21, 10, 11);
        let filename = "fighting_kittens.jpg";
        let mt = mime::IMAGE_JPEG;
        let actual = new_asset_id(import_date, Path::new(filename), &mt);
        // Cannot compare the actual value because it incorporates a random
        // number, can only decode and check the basic format matches
        // expectations.
        let decoded = general_purpose::STANDARD.decode(&actual).unwrap();
        let as_string = std::str::from_utf8(&decoded).unwrap();
        assert!(as_string.starts_with("2018/05/31/2100/"));
        assert!(as_string.ends_with(".jpg"));
        assert_eq!(as_string.len(), 46);

        // test with an image/jpeg asset with an incorrect extension
        let filename = "fighting_kittens.foo";
        let actual = new_asset_id(import_date, Path::new(filename), &mt);
        let decoded = general_purpose::STANDARD.decode(&actual).unwrap();
        let as_string = std::str::from_utf8(&decoded).unwrap();
        // jpe because it's first in the list of [jpe, jpeg, jpg]
        assert!(as_string.ends_with(".foo.jpe"));

        // test with an image/jpeg asset with _no_ extension
        let filename = "fighting_kittens";
        let actual = new_asset_id(import_date, Path::new(filename), &mt);
        let decoded = general_purpose::STANDARD.decode(&actual).unwrap();
        let as_string = std::str::from_utf8(&decoded).unwrap();
        // jpe because it's first in the list of [jpe, jpeg, jpg]
        assert!(as_string.ends_with(".jpe"));
    }

    #[test]
    fn test_get_file_name() {
        let filepath = Path::new("./tests/fixtures/fighting_kittens.jpg");
        let actual = get_file_name(&filepath);
        assert_eq!(actual, "fighting_kittens.jpg");
    }

    #[test]
    fn test_import_asset_new() {
        // arrange
        let digest = "sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07";
        let infile = PathBuf::from("./tests/fixtures/dcp_1069.jpg");
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
        let usecase = ImportAsset::new(Arc::new(records), Arc::new(blobs));
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
        let infile = PathBuf::from("./tests/fixtures/dcp_1069.jpg");
        let infile_copy = infile.clone();
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
        let mut blobs = MockBlobRepository::new();
        blobs
            .expect_store_blob()
            .with(function(move |p| p == infile_copy.as_path()), always())
            .returning(|_, _| Ok(()));
        // act
        let usecase = ImportAsset::new(Arc::new(records), Arc::new(blobs));
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
    fn test_get_dimensions() {
        let filename = "./tests/fixtures/dcp_1069.jpg";
        let mt = mime::IMAGE_JPEG;
        let filepath = Path::new(filename);
        let actual = get_dimensions(&mt, filepath);
        assert!(actual.is_ok());
        let dim = actual.unwrap();
        assert_eq!(dim.0, 440);
        assert_eq!(dim.1, 292);

        // rotated sideways (dimensions are flipped)
        let filename = "./tests/fixtures/fighting_kittens.jpg";
        let filepath = Path::new(filename);
        let actual = get_dimensions(&mt, filepath);
        assert!(actual.is_ok());
        let dim = actual.unwrap();
        assert_eq!(dim.0, 384);
        assert_eq!(dim.1, 512);

        let filename = "./tests/fixtures/animal-cat-cute-126407.jpg";
        let filepath = Path::new(filename);
        let actual = get_dimensions(&mt, filepath);
        assert!(actual.is_ok());
        let dim = actual.unwrap();
        assert_eq!(dim.0, 2067);
        assert_eq!(dim.1, 1163);

        // not an image
        let filename = "./tests/fixtures/lorem-ipsum.txt";
        let filepath = Path::new(filename);
        let actual = get_dimensions(&mt, filepath);
        assert!(actual.is_err());
    }
}
