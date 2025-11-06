//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::repositories::BlobRepository;
use crate::domain::repositories::RecordRepository;
use crate::domain::usecases::NoParams;
use anyhow::Error;
use log::{info, warn};
use std::fs;
use std::io;
use std::path::Path;

pub struct Analyze {
    records: Box<dyn RecordRepository>,
    blobs: Box<dyn BlobRepository>,
}

impl Analyze {
    pub fn new(records: Box<dyn RecordRepository>, blobs: Box<dyn BlobRepository>) -> Self {
        Self { records, blobs }
    }

    fn examine_image(&self, blob_path: &Path, counts: &mut Counts) -> Result<(), Error> {
        counts.is_image += 1;
        let file = fs::File::open(blob_path)?;
        let mut buffer = io::BufReader::new(&file);
        let reader = exif::Reader::new();
        if let Ok(exif) = reader.read_from_container(&mut buffer) {
            counts.has_exif_data += 1;
            if exif
                .get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY)
                .is_some()
            {
                counts.has_original_datetime += 1;
            }
            if exif
                .get_field(exif::Tag::OffsetTimeOriginal, exif::In::PRIMARY)
                .is_some()
            {
                counts.has_original_timezone += 1;
            }
            // n.b. not all images have GPSVersionID or the many other GPS
            // related files, but if it is missing the latitude, then it
            // certainly cannot have useful information for our purposes
            if exif
                .get_field(exif::Tag::GPSLatitude, exif::In::PRIMARY)
                .is_some()
            {
                counts.has_gps_coords += 1;
            }
        }
        Ok(())
    }
}

impl super::UseCase<Counts, NoParams> for Analyze {
    fn call(&self, _params: NoParams) -> Result<Counts, Error> {
        let mut counts: Counts = Default::default();
        // raise any database errors immediately
        let all_assets = self.records.all_assets()?;
        for asset_id in all_assets {
            info!("checking asset {}", asset_id);
            if let Ok(blob_path) = self.blobs.blob_path(&asset_id) {
                counts.total_assets += 1;
                if blob_path.exists() {
                    let asset = self.records.get_asset_by_id(&asset_id)?;
                    if let Ok(media_type) = asset.media_type.parse::<mime::Mime>() {
                        if media_type.type_() == mime::IMAGE {
                            self.examine_image(&blob_path, &mut counts)?;
                        } else if media_type.type_() == mime::VIDEO {
                            counts.is_video += 1;
                        }
                    }
                } else {
                    warn!("file missing for asset {}", asset_id);
                    counts.missing_files += 1;
                }
            }
        }
        info!("analysis complete");
        Ok(counts)
    }
}

/// Summary of the results of analyzing all of the assets.
#[derive(Debug, Default)]
pub struct Counts {
    /// Total number of assets in the database.
    pub total_assets: u32,
    /// Number of assets for which the file is missing.
    pub missing_files: u32,
    /// Number of assets that represent an image.
    pub is_image: u32,
    /// Number of assets that represent a video.
    pub is_video: u32,
    /// Number of images that have Exif data.
    pub has_exif_data: u32,
    /// Number images that have GPS coordinates.
    pub has_gps_coords: u32,
    /// Number images that have an original date/time.
    pub has_original_datetime: u32,
    /// Number images that have an original time zone.
    pub has_original_timezone: u32,
}

#[cfg(test)]
mod tests {
    use super::super::UseCase;
    use super::*;
    use crate::domain::entities::Asset;
    use crate::domain::repositories::MockBlobRepository;
    use crate::domain::repositories::MockRecordRepository;
    use chrono::prelude::*;
    use hashed_array_tree::{hat, HashedArrayTree};
    use mockall::predicate::*;
    use std::path::PathBuf;

    #[test]
    fn test_analyze_single_image() {
        // arrange
        let asset1_id = "dGVzdHMvZml4dHVyZXMvZmlnaHRpbmdfa2l0dGVucy5qcGc=";
        let digest1 = "sha256-82084759e4c766e94bb91d8cf9ed9edc1d4480025205f5109ec39a806509ee09";
        let mut records = MockRecordRepository::new();
        records
            .expect_all_assets()
            .returning(move || Ok(hat![asset1_id.to_owned()]));
        records
            .expect_get_asset_by_id()
            .with(eq(asset1_id))
            .returning(move |_| {
                Ok(Asset {
                    key: asset1_id.to_owned(),
                    checksum: digest1.to_owned(),
                    filename: "fighting_kittens.jpg".to_owned(),
                    byte_length: 39932,
                    media_type: "image/jpeg".to_owned(),
                    tags: vec!["kitten".to_owned()],
                    import_date: Utc::now(),
                    caption: None,
                    location: None,
                    user_date: None,
                    original_date: None,
                    dimensions: None,
                })
            });
        let mut blobs = MockBlobRepository::new();
        blobs
            .expect_blob_path()
            .with(eq(asset1_id))
            .returning(|_| Ok(PathBuf::from("tests/fixtures/fighting_kittens.jpg")));
        // act
        let usecase = Analyze::new(Box::new(records), Box::new(blobs));
        let params = NoParams {};
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let counts = result.unwrap();
        assert_eq!(counts.total_assets, 1);
        assert_eq!(counts.missing_files, 0);
        assert_eq!(counts.is_image, 1);
        assert_eq!(counts.is_video, 0);
        assert_eq!(counts.has_exif_data, 1);
        assert_eq!(counts.has_gps_coords, 0);
        assert_eq!(counts.has_original_datetime, 0);
        assert_eq!(counts.has_original_timezone, 0);
    }

    #[test]
    fn test_diagnose_multiple_assets() {
        // arrange
        let asset1_id = "dGVzdHMvZml4dHVyZXMvbm9fc3VjaF9maWxlLmpwZw==";
        let digest1 = "sha256-82084759e4c766e94bb91d8cf9ed9edc1d4480025205f5109ec39a806509ee09";
        let asset2_id = "dGVzdHMvZml4dHVyZXMvZmlnaHRpbmdfa2l0dGVucy5qcGc=";
        let digest2 = "sha256-82084759e4c766e94bb91d8cf9ed9edc1d4480025205f5109ec39a806509ee09";
        let asset3_id = "dGVzdHMvZml4dHVyZXMvSU1HXzAzODUuSlBH";
        let digest3 = "sha256-d020066fd41970c2eebc51b1e712a500de4966cef0daf4890dc238d80cbaebb2";
        let mut records = MockRecordRepository::new();
        records.expect_all_assets().returning(move || {
            Ok(hat![
                asset1_id.to_owned(),
                asset2_id.to_owned(),
                asset3_id.to_owned(),
            ])
        });
        records
            .expect_get_asset_by_id()
            .with(eq(asset1_id))
            .returning(move |_| {
                Ok(Asset {
                    key: asset1_id.to_owned(),
                    checksum: digest1.to_owned(),
                    filename: "no_such_file.jpg".to_owned(),
                    byte_length: 0,
                    media_type: "image/jpeg".to_owned(),
                    tags: vec![],
                    import_date: Utc::now(),
                    caption: None,
                    location: None,
                    user_date: None,
                    original_date: None,
                    dimensions: None,
                })
            });
        records
            .expect_get_asset_by_id()
            .with(eq(asset2_id))
            .returning(move |_| {
                Ok(Asset {
                    key: asset2_id.to_owned(),
                    checksum: digest2.to_owned(),
                    filename: "fighting_kittens.jpg".to_owned(),
                    byte_length: 39932,
                    media_type: "image/jpeg".to_owned(),
                    tags: vec!["kitten".to_owned()],
                    import_date: Utc::now(),
                    caption: None,
                    location: None,
                    user_date: None,
                    original_date: None,
                    dimensions: None,
                })
            });
        records
            .expect_get_asset_by_id()
            .with(eq(asset3_id))
            .returning(move |_| {
                Ok(Asset {
                    key: asset3_id.to_owned(),
                    checksum: digest3.to_owned(),
                    filename: "IMG_0385.JPG".to_owned(),
                    byte_length: 59908,
                    media_type: "image/jpeg".to_owned(),
                    tags: vec!["coaster".to_owned()],
                    import_date: Utc::now(),
                    caption: None,
                    location: None,
                    user_date: None,
                    original_date: None,
                    dimensions: None,
                })
            });
        let mut blobs = MockBlobRepository::new();
        blobs
            .expect_blob_path()
            .with(eq(asset1_id))
            .returning(|_| Ok(PathBuf::from("tests/fixtures/no_such_file.jpg")));
        blobs
            .expect_blob_path()
            .with(eq(asset2_id))
            .returning(|_| Ok(PathBuf::from("tests/fixtures/fighting_kittens.jpg")));
        blobs
            .expect_blob_path()
            .with(eq(asset3_id))
            .returning(|_| Ok(PathBuf::from("tests/fixtures/IMG_0385.JPG")));
        // act
        let usecase = Analyze::new(Box::new(records), Box::new(blobs));
        let params = NoParams {};
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let counts = result.unwrap();
        assert_eq!(counts.total_assets, 3);
        assert_eq!(counts.missing_files, 1);
        assert_eq!(counts.is_image, 2);
        assert_eq!(counts.is_video, 0);
        assert_eq!(counts.has_exif_data, 2);
        assert_eq!(counts.has_gps_coords, 1);
        assert_eq!(counts.has_original_datetime, 1);
        assert_eq!(counts.has_original_timezone, 1);
    }
}
