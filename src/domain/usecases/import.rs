//
// Copyright (c) 2025 Nathan Fiedler
//
use crate::domain::entities::Asset;
use crate::domain::repositories::{
    BlobRepository, LocationRepository, RecordRepository, SearchRepository,
};
use crate::domain::usecases::{checksum_file, get_gps_coordinates, get_original_date};
use anyhow::Error;
use chrono::prelude::*;
use log::error;
use std::cmp;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

pub struct ImportAsset {
    records: Arc<dyn RecordRepository>,
    cache: Arc<dyn SearchRepository>,
    blobs: Arc<dyn BlobRepository>,
    geocoder: Arc<dyn LocationRepository>,
}

impl ImportAsset {
    pub fn new(
        records: Arc<dyn RecordRepository>,
        cache: Arc<dyn SearchRepository>,
        blobs: Arc<dyn BlobRepository>,
        geocoder: Arc<dyn LocationRepository>,
    ) -> Self {
        Self {
            records,
            cache,
            blobs,
            geocoder,
        }
    }

    // Create an asset entity based on available information.
    fn create_asset(&self, digest: String, params: Params) -> Result<Asset, Error> {
        let now = Utc::now();
        let asset_id = super::new_asset_id(now, &params.filepath, &params.media_type);
        let filename = super::get_file_name(&params.filepath);
        let metadata = std::fs::metadata(&params.filepath)?;
        let byte_length = metadata.len();
        let location =
            match get_gps_coordinates(&params.media_type, &params.filepath) { Ok(coords) => {
                match self.geocoder.find_location(&coords) {
                    Ok(geoloc) => Some(super::convert_location(geoloc)),
                    Err(err) => {
                        error!("import: geocode error: {}", err);
                        None
                    }
                }
            } _ => {
                None
            }};
        // some applications will set the file modified time appropriately, so
        // if the asset itself does not have an original date/time, use that
        let original_date = get_original_date(&params.media_type, &params.filepath)
            .ok()
            .or(params.last_modified);
        let dimensions = super::get_dimensions(&params.media_type, &params.filepath).ok();
        let asset = Asset {
            key: asset_id,
            checksum: digest,
            filename,
            byte_length,
            media_type: params.media_type.to_string(),
            tags: vec![],
            import_date: now,
            caption: None,
            location,
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
                self.cache.clear()?;
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
    last_modified: Option<DateTime<Utc>>,
}

impl Params {
    pub fn new(
        filepath: PathBuf,
        media_type: mime::Mime,
        last_modified: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            filepath,
            media_type,
            last_modified,
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

#[cfg(test)]
mod tests {
    use super::super::{new_asset_id, UseCase};
    use super::*;
    use crate::domain::entities::GeocodedLocation;
    use crate::domain::repositories::MockBlobRepository;
    use crate::domain::repositories::MockLocationRepository;
    use crate::domain::repositories::MockRecordRepository;
    use crate::domain::repositories::MockSearchRepository;
    use base64::{engine::general_purpose, Engine as _};
    use mockall::predicate::*;
    use std::path::Path;

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
        assert!(as_string.ends_with(".foo.jpeg"));

        // test with an image/jpeg asset with _no_ extension
        let filename = "fighting_kittens";
        let actual = new_asset_id(import_date, Path::new(filename), &mt);
        let decoded = general_purpose::STANDARD.decode(&actual).unwrap();
        let as_string = std::str::from_utf8(&decoded).unwrap();
        assert!(as_string.ends_with(".jpeg"));
    }

    #[test]
    fn test_import_asset_new() {
        // arrange
        let digest = "sha256-d020066fd41970c2eebc51b1e712a500de4966cef0daf4890dc238d80cbaebb2";
        // use an asset that has GPS coordinates in the Exif data to trigger the
        // geocoder and result in a meaningful location value
        let infile = PathBuf::from("./tests/fixtures/IMG_0385.JPG");
        let infile_copy = infile.clone();
        let mut records = MockRecordRepository::new();
        records
            .expect_get_asset_by_digest()
            .with(eq(digest))
            .returning(|_| Ok(None));
        records.expect_put_asset().returning(|_| Ok(()));
        let mut cache = MockSearchRepository::new();
        cache.expect_clear().returning(|| Ok(()));
        let mut blobs = MockBlobRepository::new();
        blobs
            .expect_store_blob()
            .with(function(move |p| p == infile_copy.as_path()), always())
            .returning(|_, _| Ok(()));
        let mut geocoder = MockLocationRepository::new();
        geocoder.expect_find_location().returning(|_| {
            Ok(GeocodedLocation {
                city: Some("Yao".into()),
                region: Some("Osaka".into()),
                country: Some("Japan".into()),
            })
        });
        // act
        let usecase = ImportAsset::new(
            Arc::new(records),
            Arc::new(cache),
            Arc::new(blobs),
            Arc::new(geocoder),
        );
        let media_type = mime::IMAGE_JPEG;
        let params = Params::new(infile, media_type, None);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let asset = result.unwrap();
        assert_eq!(asset.checksum, digest);
        assert_eq!(asset.filename, "IMG_0385.JPG");
        assert_eq!(asset.byte_length, 59908);
        assert_eq!(asset.media_type, "image/jpeg");
        assert!(asset.tags.is_empty());
        assert!(asset.location.is_some());
        let location = asset.location.unwrap();
        assert_eq!(location.city.as_ref().unwrap(), "Yao");
        assert_eq!(location.region.as_ref().unwrap(), "Osaka");
        assert!(asset.original_date.is_some(), "expected an original date");
        assert_eq!(asset.original_date.unwrap().year(), 2024);
        assert!(asset.dimensions.is_some(), "expected image dimensions");
        assert_eq!(asset.dimensions.as_ref().unwrap().0, 302);
        assert_eq!(asset.dimensions.as_ref().unwrap().1, 403);
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
        let mut cache = MockSearchRepository::new();
        cache.expect_clear().never();
        let mut blobs = MockBlobRepository::new();
        blobs
            .expect_store_blob()
            .with(function(move |p| p == infile_copy.as_path()), always())
            .returning(|_, _| Ok(()));
        let mut geocoder = MockLocationRepository::new();
        geocoder
            .expect_find_location()
            .returning(|_| Ok(Default::default()));
        // act
        let usecase = ImportAsset::new(
            Arc::new(records),
            Arc::new(cache),
            Arc::new(blobs),
            Arc::new(geocoder),
        );
        let media_type = mime::IMAGE_JPEG;
        let params = Params::new(infile, media_type, None);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let asset = result.unwrap();
        assert_eq!(asset.checksum, digest);
        assert_eq!(asset.filename, "dcp_1069.jpg");
    }
}
