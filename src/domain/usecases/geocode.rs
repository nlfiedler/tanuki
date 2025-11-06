//
// Copyright (c) 2025 Nathan Fiedler
//
use crate::domain::entities::Location;
use crate::domain::repositories::{
    BlobRepository, LocationRepository, RecordRepository, SearchRepository,
};
use crate::domain::usecases::get_gps_coordinates;
use anyhow::Error;
use log::{error, info, trace, warn};
use std::cmp;
use std::fmt;
use std::sync::Arc;

///
/// Scan all assets and consider those images that have GPS coordinates in their
/// Exif data. If the asset record does not have city and region defined, then
/// invoke a reverse geocoding API with the GPS coordinates in the hopes of
/// finding a match. If such a match is found, then add that information to the
/// asset record such that it will have values for city and region, in addition
/// to whatever location label was already specified.
///
pub struct Geocoder {
    records: Box<dyn RecordRepository>,
    blobs: Box<dyn BlobRepository>,
    geocoder: Arc<dyn LocationRepository>,
    cache: Box<dyn SearchRepository>,
}

impl Geocoder {
    pub fn new(
        records: Box<dyn RecordRepository>,
        blobs: Box<dyn BlobRepository>,
        geocoder: Arc<dyn LocationRepository>,
        cache: Box<dyn SearchRepository>,
    ) -> Self {
        Self {
            records,
            blobs,
            geocoder,
            cache,
        }
    }
}

impl super::UseCase<u64, Params> for Geocoder {
    fn call(&self, params: Params) -> Result<u64, Error> {
        let mut fixed_count: u64 = 0;
        // raise any database errors immediately
        let all_assets = self.records.all_assets()?;
        for asset_id in all_assets {
            info!("checking asset {}", asset_id);
            let mut asset = self.records.get_asset_by_id(&asset_id)?;
            // consider those assets which have a media type that can be parsed,
            // have a valid blob path, GPS coordinates can be read from the
            // asset, and a location can be found for those GPS coordinates
            if params.overwrite
                || asset
                    .location
                    .as_ref()
                    .is_none_or(|l| l.city.is_none() && l.region.is_none())
            {
                if let Ok(media_type) = asset.media_type.parse::<mime::Mime>() {
                    match self.blobs.blob_path(&asset_id) {
                        Ok(blob_path) => {
                            if let Ok(coords) = get_gps_coordinates(&media_type, &blob_path) {
                                trace!("asset has GPS coordinates");
                                match self.geocoder.find_location(&coords) {
                                    Ok(geoloc) => {
                                        let found_loc = super::convert_location(geoloc);
                                        // ensure the geocoder returned a meaningful result
                                        if found_loc.city.is_some() || found_loc.region.is_some() {
                                            trace!("successfully converted to domain location");
                                            if let Some(old_loc) = asset.location.as_ref() {
                                                // fill in city/region as appropriate
                                                asset.location = Some(Location {
                                                    label: old_loc.label.clone(),
                                                    city: found_loc.city,
                                                    region: found_loc.region,
                                                });
                                                self.records.put_asset(&asset)?;
                                                fixed_count += 1;
                                                info!("merged existing location values");
                                            } else {
                                                // the asset has no location at all, fix it
                                                asset.location = Some(found_loc);
                                                self.records.put_asset(&asset)?;
                                                fixed_count += 1;
                                                info!("assigned new location values");
                                            }
                                        }
                                    }
                                    Err(err) => error!("replace: geocode error: {}", err),
                                }
                            }
                        }
                        _ => {
                            warn!("could not get path for asset {}", asset_id);
                        }
                    }
                } else {
                    warn!("could not parse media type for asset {}", asset_id);
                }
            }
        }
        info!("analysis complete, fixed {} assets", fixed_count);
        if fixed_count > 0 {
            self.cache.clear()?;
        }
        Ok(fixed_count)
    }
}

pub struct Params {
    /// If true, replace whatever city and region may be present.
    overwrite: bool,
}

impl Params {
    pub fn new(overwrite: bool) -> Self {
        Self { overwrite }
    }
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Params({})", self.overwrite)
    }
}

impl cmp::PartialEq for Params {
    fn eq(&self, other: &Self) -> bool {
        self.overwrite == other.overwrite
    }
}

impl cmp::Eq for Params {}

#[cfg(test)]
mod tests {
    use super::super::UseCase;
    use super::*;
    use crate::domain::entities::{Asset, GeocodedLocation};
    use crate::domain::repositories::MockBlobRepository;
    use crate::domain::repositories::MockLocationRepository;
    use crate::domain::repositories::MockRecordRepository;
    use crate::domain::repositories::MockSearchRepository;
    use chrono::prelude::*;
    use hashed_array_tree::{hat, HashedArrayTree};
    use mockall::predicate::*;
    use std::path::PathBuf;

    #[test]
    fn test_geocode_asset_location_unavailable() {
        // arrange
        let asset1_id = "dGVzdHMvZml4dHVyZXMvSU1HXzAzODUuSlBH";
        let digest1 = "sha256-d020066fd41970c2eebc51b1e712a500de4966cef0daf4890dc238d80cbaebb2";
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
            .returning(|_| Ok(PathBuf::from("tests/fixtures/IMG_0385.JPG")));
        let mut geocoder = MockLocationRepository::new();
        geocoder.expect_find_location().returning(|_| {
            Ok(GeocodedLocation {
                city: None,
                region: None,
                country: None,
            })
        });
        let mut cache = MockSearchRepository::new();
        cache.expect_clear().never();
        // act
        let usecase = Geocoder::new(
            Box::new(records),
            Box::new(blobs),
            Arc::new(geocoder),
            Box::new(cache),
        );
        let params = Params::new(false);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_geocode_asset_without_location() {
        // arrange
        let asset1_id = "dGVzdHMvZml4dHVyZXMvSU1HXzAzODUuSlBH";
        let digest1 = "sha256-d020066fd41970c2eebc51b1e712a500de4966cef0daf4890dc238d80cbaebb2";
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
        let expected_loc = Some(Location {
            label: None,
            city: Some("Yao".into()),
            region: Some("Osaka".into()),
        });
        records
            .expect_put_asset()
            .withf(move |asset| asset.key == asset1_id && asset.location == expected_loc)
            .returning(|_| Ok(()));
        let mut blobs = MockBlobRepository::new();
        blobs
            .expect_blob_path()
            .with(eq(asset1_id))
            .returning(|_| Ok(PathBuf::from("tests/fixtures/IMG_0385.JPG")));
        let mut geocoder = MockLocationRepository::new();
        geocoder.expect_find_location().returning(|_| {
            Ok(GeocodedLocation {
                city: Some("Yao".into()),
                region: Some("Osaka".into()),
                country: Some("Japan".into()),
            })
        });
        let mut cache = MockSearchRepository::new();
        cache.expect_clear().returning(|| Ok(()));
        // act
        let usecase = Geocoder::new(
            Box::new(records),
            Box::new(blobs),
            Arc::new(geocoder),
            Box::new(cache),
        );
        let params = Params::new(false);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_geocode_asset_location_labeled() {
        // arrange
        let asset1_id = "dGVzdHMvZml4dHVyZXMvSU1HXzAzODUuSlBH";
        let digest1 = "sha256-d020066fd41970c2eebc51b1e712a500de4966cef0daf4890dc238d80cbaebb2";
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
                    filename: "IMG_0385.JPG".to_owned(),
                    byte_length: 59908,
                    media_type: "image/jpeg".to_owned(),
                    tags: vec!["coaster".to_owned()],
                    import_date: Utc::now(),
                    caption: None,
                    location: Some(Location {
                        label: Some("my desk".into()),
                        city: None,
                        region: None,
                    }),
                    user_date: None,
                    original_date: None,
                    dimensions: None,
                })
            });
        let expected_loc = Some(Location {
            label: Some("my desk".into()),
            city: Some("Yao".into()),
            region: Some("Osaka".into()),
        });
        records
            .expect_put_asset()
            .withf(move |asset| asset.key == asset1_id && asset.location == expected_loc)
            .returning(|_| Ok(()));
        let mut blobs = MockBlobRepository::new();
        blobs
            .expect_blob_path()
            .with(eq(asset1_id))
            .returning(|_| Ok(PathBuf::from("tests/fixtures/IMG_0385.JPG")));
        let mut geocoder = MockLocationRepository::new();
        geocoder.expect_find_location().returning(|_| {
            Ok(GeocodedLocation {
                city: Some("Yao".into()),
                region: Some("Osaka".into()),
                country: Some("Japan".into()),
            })
        });
        let mut cache = MockSearchRepository::new();
        cache.expect_clear().returning(|| Ok(()));
        // act
        let usecase = Geocoder::new(
            Box::new(records),
            Box::new(blobs),
            Arc::new(geocoder),
            Box::new(cache),
        );
        let params = Params::new(false);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_geocode_asset_complete_location() {
        // arrange
        let asset1_id = "dGVzdHMvZml4dHVyZXMvSU1HXzAzODUuSlBH";
        let digest1 = "sha256-d020066fd41970c2eebc51b1e712a500de4966cef0daf4890dc238d80cbaebb2";
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
                    filename: "IMG_0385.JPG".to_owned(),
                    byte_length: 59908,
                    media_type: "image/jpeg".to_owned(),
                    tags: vec!["coaster".to_owned()],
                    import_date: Utc::now(),
                    caption: None,
                    location: Some(Location {
                        label: Some("my desk".into()),
                        city: Some("Oakland".into()),
                        region: Some("CA".into()),
                    }),
                    user_date: None,
                    original_date: None,
                    dimensions: None,
                })
            });
        let mut blobs = MockBlobRepository::new();
        blobs
            .expect_blob_path()
            .with(eq(asset1_id))
            .returning(|_| Ok(PathBuf::from("tests/fixtures/IMG_0385.JPG")));
        let mut geocoder = MockLocationRepository::new();
        geocoder.expect_find_location().returning(|_| {
            Ok(GeocodedLocation {
                city: Some("Yao".into()),
                region: Some("Osaka".into()),
                country: Some("Japan".into()),
            })
        });
        let mut cache = MockSearchRepository::new();
        cache.expect_clear().returning(|| Ok(()));
        // act
        let usecase = Geocoder::new(
            Box::new(records),
            Box::new(blobs),
            Arc::new(geocoder),
            Box::new(cache),
        );
        let params = Params::new(false);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_geocode_asset_overwrite_location() {
        // arrange
        let asset1_id = "dGVzdHMvZml4dHVyZXMvSU1HXzAzODUuSlBH";
        let digest1 = "sha256-d020066fd41970c2eebc51b1e712a500de4966cef0daf4890dc238d80cbaebb2";
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
                    filename: "IMG_0385.JPG".to_owned(),
                    byte_length: 59908,
                    media_type: "image/jpeg".to_owned(),
                    tags: vec!["coaster".to_owned()],
                    import_date: Utc::now(),
                    caption: None,
                    location: Some(Location {
                        label: Some("my desk".into()),
                        city: Some("Oakland".into()),
                        region: Some("CA".into()),
                    }),
                    user_date: None,
                    original_date: None,
                    dimensions: None,
                })
            });
        let expected_loc = Some(Location {
            label: Some("my desk".into()),
            city: Some("Yao".into()),
            region: Some("Osaka".into()),
        });
        records
            .expect_put_asset()
            .withf(move |asset| asset.key == asset1_id && asset.location == expected_loc)
            .returning(|_| Ok(()));
        let mut blobs = MockBlobRepository::new();
        blobs
            .expect_blob_path()
            .with(eq(asset1_id))
            .returning(|_| Ok(PathBuf::from("tests/fixtures/IMG_0385.JPG")));
        let mut geocoder = MockLocationRepository::new();
        geocoder.expect_find_location().returning(|_| {
            Ok(GeocodedLocation {
                city: Some("Yao".into()),
                region: Some("Osaka".into()),
                country: Some("Japan".into()),
            })
        });
        let mut cache = MockSearchRepository::new();
        cache.expect_clear().returning(|| Ok(()));
        // act
        let usecase = Geocoder::new(
            Box::new(records),
            Box::new(blobs),
            Arc::new(geocoder),
            Box::new(cache),
        );
        let params = Params::new(true);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 1);
    }
}
