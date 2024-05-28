//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::Asset;
use crate::domain::repositories::RecordRepository;
use crate::domain::repositories::{BlobRepository, LocationRepository};
use crate::domain::usecases::{checksum_file, get_gps_coordinates, get_original_date};
use anyhow::Error;
use chrono::prelude::*;
use std::cmp;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

///
/// Replaces the record and blob associated with an existing asset and update
/// all of the appropriate fields in the asset record. Because the file
/// extension may have changed, an entirely new identifier is generated and a
/// new record is created.
///
pub struct ReplaceAsset {
    records: Arc<dyn RecordRepository>,
    blobs: Arc<dyn BlobRepository>,
    geocoder: Arc<dyn LocationRepository>,
}

impl ReplaceAsset {
    pub fn new(
        records: Arc<dyn RecordRepository>,
        blobs: Arc<dyn BlobRepository>,
        geocoder: Arc<dyn LocationRepository>,
    ) -> Self {
        Self {
            records,
            blobs,
            geocoder,
        }
    }

    // Update an asset entity based on available information.
    fn update_asset(&self, digest: String, params: Params) -> Result<Asset, Error> {
        let mut asset = self.records.get_asset(&params.asset_id)?;
        asset.checksum = digest;
        asset.filename = super::get_file_name(&params.filepath);
        asset.media_type = params.media_type.to_string();
        let metadata = std::fs::metadata(&params.filepath)?;
        asset.byte_length = metadata.len();
        if let Some(coords) = get_gps_coordinates(&params.media_type, &params.filepath).ok() {
            let converted = super::convert_location(self.geocoder.find_location(&coords).ok());
            asset.location = super::merge_locations(asset.location.clone(), converted);
        }
        asset.original_date = get_original_date(&params.media_type, &params.filepath).ok();
        asset.dimensions = super::get_dimensions(&params.media_type, &params.filepath).ok();
        Ok(asset)
    }
}

impl super::UseCase<Asset, Params> for ReplaceAsset {
    fn call(&self, params: Params) -> Result<Asset, Error> {
        let digest = checksum_file(&params.filepath)?;
        let asset = match self.records.get_asset_by_digest(&digest)? {
            Some(_) => {
                // if an identical asset already exists, then replace is not
                // possible and we simply need to remove the uploaded file
                std::fs::remove_file(&params.filepath)?;
                // return the original record as-is so the client can know that
                // nothing changed on the backend
                self.records.get_asset(&params.asset_id)?
            }
            None => {
                let mut asset = self.update_asset(digest, params.clone())?;
                let old_asset_id = asset.key.clone();
                let now = Utc::now();
                let new_asset_id = super::new_asset_id(now, &params.filepath, &params.media_type);
                asset.key = new_asset_id.clone();
                self.records.put_asset(&asset)?;
                self.blobs.rename_blob(&old_asset_id, &new_asset_id)?;
                // blob repo will ensure the temporary file is (re)moved
                self.blobs.replace_blob(&params.filepath, &asset)?;
                self.records.delete_asset(&old_asset_id)?;
                // clearing the cache is just an optimization since the new
                // asset identifier has been generated and the client should not
                // be requesting a thumbnail using the old identifier
                self.blobs.clear_cache(&asset.key)?;
                asset
            }
        };
        Ok(asset)
    }
}

#[derive(Clone)]
pub struct Params {
    /// Identifier of the asset to be replaced.
    asset_id: String,
    /// Path of the new file that will replace the asset.
    filepath: PathBuf,
    /// Media type for the new file.
    media_type: mime::Mime,
}

impl Params {
    pub fn new(asset_id: String, filepath: PathBuf, media_type: mime::Mime) -> Self {
        Self {
            asset_id,
            filepath,
            media_type,
        }
    }
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Params({:?})", self.asset_id)
    }
}

impl cmp::PartialEq for Params {
    fn eq(&self, other: &Self) -> bool {
        self.asset_id == other.asset_id
    }
}

impl cmp::Eq for Params {}

#[cfg(test)]
mod tests {
    use super::super::UseCase;
    use super::*;
    use crate::domain::entities::Location;
    use crate::domain::repositories::MockBlobRepository;
    use crate::domain::repositories::MockLocationRepository;
    use crate::domain::repositories::MockRecordRepository;
    use mockall::predicate::*;
    use tempfile::tempdir;

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
    fn test_replace_asset_checksum() {
        // arrange
        let existing_asset_id = "Li90ZXN0cy9maXh0dXJlcy9JTUdfMDM4NS5KUEc=";
        let existing_digest =
            "sha256-d020066fd41970c2eebc51b1e712a500de4966cef0daf4890dc238d80cbaebb2";
        let unchanged_asset_id = "Li90ZXN0cy9maXh0dXJlcy9maWdodGluZ19raXR0ZW5zLmpwZw==";
        let unchanged_digest =
            "sha256-82084759e4c766e94bb91d8cf9ed9edc1d4480025205f5109ec39a806509ee09";
        // copy test file to temporary path as it will be (re)moved
        let tmpdir = tempdir().unwrap();
        let original = PathBuf::from("./tests/fixtures/IMG_0385.JPG");
        let infile = tmpdir.path().join("IMG_0385.JPG");
        std::fs::copy(original, &infile).unwrap();
        let mut records = MockRecordRepository::new();
        records
            .expect_get_asset_by_digest()
            .with(eq(existing_digest))
            .returning(move |_| {
                Ok(Some(Asset {
                    key: existing_asset_id.to_owned(),
                    checksum: existing_digest.to_owned(),
                    filename: "IMG_0385.JPG".to_owned(),
                    byte_length: 59908,
                    media_type: "image/jpeg".to_owned(),
                    tags: vec!["cute".to_owned()],
                    import_date: Utc::now(),
                    caption: None,
                    location: Some(Location::new("hawaii")),
                    user_date: None,
                    original_date: None,
                    dimensions: None,
                }))
            });
        records
            .expect_get_asset()
            .with(eq(unchanged_asset_id))
            .returning(move |_| {
                Ok(Asset {
                    key: unchanged_asset_id.to_owned(),
                    checksum: unchanged_digest.to_owned(),
                    filename: "fighting_kittens.jpg".to_owned(),
                    byte_length: 39932,
                    media_type: "image/jpeg".to_owned(),
                    tags: vec!["cows".to_owned()],
                    import_date: Utc::now(),
                    caption: None,
                    location: Some(Location::new("hawaii")),
                    user_date: None,
                    original_date: None,
                    dimensions: None,
                })
            });
        let blobs = MockBlobRepository::new();
        let geocoder = MockLocationRepository::new();
        // act
        let usecase = ReplaceAsset::new(Arc::new(records), Arc::new(blobs), Arc::new(geocoder));
        let media_type = mime::IMAGE_JPEG;
        let params = Params::new(unchanged_asset_id.to_owned(), infile, media_type);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let asset = result.unwrap();
        assert_eq!(asset.checksum, unchanged_digest);
        assert_eq!(asset.filename, "fighting_kittens.jpg");
        assert_eq!(asset.byte_length, 39932);
        assert_eq!(asset.media_type, "image/jpeg");
        assert!(asset.location.is_some());
        let location = asset.location.unwrap();
        assert_eq!(location.label.as_ref().unwrap(), "hawaii");
    }

    #[test]
    fn test_replace_asset_location() {
        // arrange
        let asset_id = "Li90ZXN0cy9maXh0dXJlcy9kY3BfMTA2OS5qcGc=";
        let digest = "sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07";
        // copy test file to temporary path as it will be (re)moved
        let tmpdir = tempdir().unwrap();
        let original = PathBuf::from("./tests/fixtures/dcp_1069.jpg");
        let infile = tmpdir.path().join("dcp_1069.jpg");
        let infile_copy = infile.clone();
        std::fs::copy(original, &infile).unwrap();
        let mut records = MockRecordRepository::new();
        let existing = Asset {
            key: asset_id.to_owned(),
            checksum: digest.to_owned(),
            filename: "fighting_kittens.jpg".to_owned(),
            byte_length: 39932,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cute".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: Some(Location {
                label: Some("beach".into()),
                city: Some("Honolulu".into()),
                region: Some("Hawaii".into()),
            }),
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        records
            .expect_get_asset_by_digest()
            .with(eq(digest))
            .returning(move |_| Ok(None));
        records
            .expect_get_asset()
            .with(eq(asset_id))
            .returning(move |_| Ok(existing.clone()));
        records.expect_put_asset().returning(|_| Ok(()));
        records
            .expect_delete_asset()
            .with(eq(asset_id))
            .returning(move |_| Ok(()));
        let mut blobs = MockBlobRepository::new();
        blobs.expect_rename_blob().returning(|_, _| Ok(()));
        blobs
            .expect_replace_blob()
            .with(function(move |p| p == infile_copy.as_path()), always())
            .returning(|_, _| Ok(()));
        blobs.expect_clear_cache().returning(|_| Ok(()));
        let mut geocoder = MockLocationRepository::new();
        geocoder
            .expect_find_location()
            .returning(|_| Ok(Default::default()));
        // act
        let usecase = ReplaceAsset::new(Arc::new(records), Arc::new(blobs), Arc::new(geocoder));
        let media_type = mime::IMAGE_JPEG;
        let params = Params::new(asset_id.to_owned(), infile, media_type);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let asset = result.unwrap();
        assert_eq!(asset.checksum, digest);
        assert_eq!(asset.filename, "dcp_1069.jpg");
        assert_eq!(asset.byte_length, 80977);
        assert_eq!(asset.media_type, "image/jpeg");
        assert!(asset.location.is_some());
        let location = asset.location.unwrap();
        assert_eq!(location.label.as_ref().unwrap(), "beach");
        assert_eq!(location.city.as_ref().unwrap(), "Honolulu");
        assert_eq!(location.region.as_ref().unwrap(), "Hawaii");
        let original_date = asset.original_date.unwrap();
        assert_eq!(original_date.year(), 2003);
        assert_eq!(original_date.month(), 9);
        assert_eq!(original_date.day(), 3);
    }

    #[test]
    fn test_replace_asset_date() {
        // arrange
        let asset_id = "Li90ZXN0cy9maXh0dXJlcy9maWdodGluZ19raXR0ZW5zLmpwZw==";
        let digest = "sha256-82084759e4c766e94bb91d8cf9ed9edc1d4480025205f5109ec39a806509ee09";
        // copy test file to temporary path as it will be (re)moved
        let tmpdir = tempdir().unwrap();
        let original = PathBuf::from("./tests/fixtures/fighting_kittens.jpg");
        let infile = tmpdir.path().join("fighting_kittens.jpg");
        let infile_copy = infile.clone();
        std::fs::copy(original, &infile).unwrap();
        let mut records = MockRecordRepository::new();
        let existing = Asset {
            key: asset_id.to_owned(),
            checksum: digest.to_owned(),
            filename: "dcp_1069.jpg".to_owned(),
            byte_length: 80977,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cow".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: None,
            user_date: None,
            original_date: Some(make_date_time(2003, 9, 3, 17, 24, 0)),
            dimensions: None,
        };
        records
            .expect_get_asset_by_digest()
            .with(eq(digest))
            .returning(move |_| Ok(None));
        records
            .expect_get_asset()
            .with(eq(asset_id))
            .returning(move |_| Ok(existing.clone()));
        records.expect_put_asset().returning(|_| Ok(()));
        records
            .expect_delete_asset()
            .with(eq(asset_id))
            .returning(move |_| Ok(()));
        let mut blobs = MockBlobRepository::new();
        blobs.expect_rename_blob().returning(|_, _| Ok(()));
        blobs
            .expect_replace_blob()
            .with(function(move |p| p == infile_copy.as_path()), always())
            .returning(|_, _| Ok(()));
        blobs.expect_clear_cache().returning(|_| Ok(()));
        let mut geocoder = MockLocationRepository::new();
        geocoder
            .expect_find_location()
            .returning(|_| Ok(Default::default()));
        // act
        let usecase = ReplaceAsset::new(Arc::new(records), Arc::new(blobs), Arc::new(geocoder));
        let media_type = mime::IMAGE_JPEG;
        let params = Params::new(asset_id.to_owned(), infile, media_type);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let asset = result.unwrap();
        assert_eq!(asset.checksum, digest);
        assert_eq!(asset.filename, "fighting_kittens.jpg");
        assert_eq!(asset.byte_length, 39932);
        assert_eq!(asset.media_type, "image/jpeg");
        assert!(asset.location.is_none());
        assert!(asset.original_date.is_none());
    }
}
