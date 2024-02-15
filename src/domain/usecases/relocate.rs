//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::Location;
use crate::domain::repositories::RecordRepository;
use anyhow::Error;
use log::info;
use std::cmp;
use std::fmt;

///
/// Allow setting city/region for those assets whose location label matches the
/// given value, and optionally clearing the label field.
///
pub struct Relocate {
    records: Box<dyn RecordRepository>,
}

impl Relocate {
    pub fn new(records: Box<dyn RecordRepository>) -> Self {
        Self { records }
    }
}

impl super::UseCase<u64, Params> for Relocate {
    fn call(&self, params: Params) -> Result<u64, Error> {
        let mut fixed_count: u64 = 0;
        // raise any database errors immediately
        let all_assets = self.records.all_assets()?;
        for asset_id in all_assets {
            info!("evaluating asset {}", asset_id);
            let mut asset = self.records.get_asset(&asset_id)?;
            let original_loc = asset.location.clone();
            if let Some(mut asset_loc) = asset.location.clone() {
                if let Some(label) = asset_loc.label {
                    let lower_label = label.to_lowercase();
                    // detect and remove redundant label values
                    let mut redundant: bool = false;
                    if let Some(city) = asset_loc.city.as_ref() {
                        if lower_label == city.to_lowercase() {
                            redundant = true;
                        }
                    }
                    if let Some(region) = asset_loc.region.as_ref() {
                        if lower_label == region.to_lowercase() {
                            redundant = true;
                        }
                    }
                    if redundant {
                        asset_loc.label = None;
                        asset.location = Some(asset_loc);
                    } else if lower_label == params.query
                        && asset_loc.city.is_none()
                        && asset_loc.region.is_none()
                    {
                        // if not redundant and the label matches the query,
                        // then update the city/region and maybe clear the label
                        let new_label = if params.clear_label {
                            None
                        } else {
                            Some(label)
                        };
                        asset.location = Some(Location {
                            label: new_label,
                            city: Some(params.city.clone()),
                            region: Some(params.region.clone()),
                        });
                    }
                }
            }
            if asset.location != original_loc {
                self.records.put_asset(&asset)?;
                fixed_count += 1;
            }
        }
        info!("relocation complete");
        Ok(fixed_count)
    }
}

#[derive(Clone)]
pub struct Params {
    /// Value to match against the existing label value.
    query: String,
    /// New value for the city field of the location.
    city: String,
    /// New value for the region field of the location.
    region: String,
    /// If true, clear the label field of the location.
    clear_label: bool,
}

impl Params {
    pub fn new<T: Into<String>>(query: T, city: T, region: T, clear_label: bool) -> Self {
        Self {
            query: query.into(),
            city: city.into(),
            region: region.into(),
            clear_label,
        }
    }
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Params({}, {}, {})", self.query, self.city, self.region)
    }
}

impl cmp::PartialEq for Params {
    fn eq(&self, other: &Self) -> bool {
        self.query == other.query && self.city == other.city && self.region == other.region
    }
}

impl cmp::Eq for Params {}

#[cfg(test)]
mod tests {
    use super::super::UseCase;
    use super::*;
    use crate::domain::entities::Asset;
    use crate::domain::repositories::MockRecordRepository;
    use chrono::prelude::*;
    use mockall::predicate::*;

    #[test]
    fn test_relocate_asset_without_location() {
        // arrange
        let asset1_id = "dGVzdHMvZml4dHVyZXMvSU1HXzAzODUuSlBH";
        let digest1 = "sha256-d020066fd41970c2eebc51b1e712a500de4966cef0daf4890dc238d80cbaebb2";
        let mut records = MockRecordRepository::new();
        records
            .expect_all_assets()
            .returning(move || Ok(vec![asset1_id.to_owned()]));
        records
            .expect_get_asset()
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
        // act
        let usecase = Relocate::new(Box::new(records));
        let params = Params::new("foo", "anytown", "state", false);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_relocate_asset_without_label() {
        // arrange
        let asset1_id = "dGVzdHMvZml4dHVyZXMvSU1HXzAzODUuSlBH";
        let digest1 = "sha256-d020066fd41970c2eebc51b1e712a500de4966cef0daf4890dc238d80cbaebb2";
        let mut records = MockRecordRepository::new();
        records
            .expect_all_assets()
            .returning(move || Ok(vec![asset1_id.to_owned()]));
        records
            .expect_get_asset()
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
                        label: None,
                        city: Some("Yao".into()),
                        region: Some("Osaka".into()),
                    }),
                    user_date: None,
                    original_date: None,
                    dimensions: None,
                })
            });
        // act
        let usecase = Relocate::new(Box::new(records));
        let params = Params::new("foo", "foocity", "barregion", false);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_relocate_label_not_match() {
        // arrange
        let asset1_id = "dGVzdHMvZml4dHVyZXMvSU1HXzAzODUuSlBH";
        let digest1 = "sha256-d020066fd41970c2eebc51b1e712a500de4966cef0daf4890dc238d80cbaebb2";
        let mut records = MockRecordRepository::new();
        records
            .expect_all_assets()
            .returning(move || Ok(vec![asset1_id.to_owned()]));
        records
            .expect_get_asset()
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
                        label: Some("home".into()),
                        city: None,
                        region: None,
                    }),
                    user_date: None,
                    original_date: None,
                    dimensions: None,
                })
            });
        // act
        let usecase = Relocate::new(Box::new(records));
        let params = Params::new("foo", "Yao", "Osaka", false);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_relocate_label_matches() {
        // arrange
        let asset1_id = "dGVzdHMvZml4dHVyZXMvSU1HXzAzODUuSlBH";
        let digest1 = "sha256-d020066fd41970c2eebc51b1e712a500de4966cef0daf4890dc238d80cbaebb2";
        let mut records = MockRecordRepository::new();
        records
            .expect_all_assets()
            .returning(move || Ok(vec![asset1_id.to_owned()]));
        records
            .expect_get_asset()
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
                        label: Some("home".into()),
                        city: None,
                        region: None,
                    }),
                    user_date: None,
                    original_date: None,
                    dimensions: None,
                })
            });
        let expected_loc = Some(Location {
            label: Some("home".into()),
            city: Some("Yao".into()),
            region: Some("Osaka".into()),
        });
        records
            .expect_put_asset()
            .withf(move |asset| asset.key == asset1_id && asset.location == expected_loc)
            .returning(|_| Ok(()));
        // act
        let usecase = Relocate::new(Box::new(records));
        let params = Params::new("home", "Yao", "Osaka", false);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_relocate_clear_redundant_city() {
        // arrange
        let asset1_id = "dGVzdHMvZml4dHVyZXMvSU1HXzAzODUuSlBH";
        let digest1 = "sha256-d020066fd41970c2eebc51b1e712a500de4966cef0daf4890dc238d80cbaebb2";
        let mut records = MockRecordRepository::new();
        records
            .expect_all_assets()
            .returning(move || Ok(vec![asset1_id.to_owned()]));
        records
            .expect_get_asset()
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
                        label: Some("yao".into()),
                        city: Some("Yao".into()),
                        region: Some("Osaka".into()),
                    }),
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
        // act
        let usecase = Relocate::new(Box::new(records));
        let params = Params::new("foobar", "Yao", "Osaka", false);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_relocate_clear_redundant_region() {
        // arrange
        let asset1_id = "dGVzdHMvZml4dHVyZXMvSU1HXzAzODUuSlBH";
        let digest1 = "sha256-d020066fd41970c2eebc51b1e712a500de4966cef0daf4890dc238d80cbaebb2";
        let mut records = MockRecordRepository::new();
        records
            .expect_all_assets()
            .returning(move || Ok(vec![asset1_id.to_owned()]));
        records
            .expect_get_asset()
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
                        label: Some("osaka".into()),
                        city: Some("Yao".into()),
                        region: Some("Osaka".into()),
                    }),
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
        // act
        let usecase = Relocate::new(Box::new(records));
        let params = Params::new("foobar", "Yao", "Osaka", false);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_relocate_clear_label() {
        // arrange
        let asset1_id = "dGVzdHMvZml4dHVyZXMvSU1HXzAzODUuSlBH";
        let digest1 = "sha256-d020066fd41970c2eebc51b1e712a500de4966cef0daf4890dc238d80cbaebb2";
        let mut records = MockRecordRepository::new();
        records
            .expect_all_assets()
            .returning(move || Ok(vec![asset1_id.to_owned()]));
        records
            .expect_get_asset()
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
                        label: Some("yaoyao".into()),
                        city: None,
                        region: None,
                    }),
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
        // act
        let usecase = Relocate::new(Box::new(records));
        let params = Params::new("yaoyao", "Yao", "Osaka", true);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_relocate_location_already_filled() {
        // arrange
        let asset1_id = "dGVzdHMvZml4dHVyZXMvSU1HXzAzODUuSlBH";
        let digest1 = "sha256-d020066fd41970c2eebc51b1e712a500de4966cef0daf4890dc238d80cbaebb2";
        let mut records = MockRecordRepository::new();
        records
            .expect_all_assets()
            .returning(move || Ok(vec![asset1_id.to_owned()]));
        records
            .expect_get_asset()
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
        // act
        let usecase = Relocate::new(Box::new(records));
        let params = Params::new("my desk", "Anytown", "AA", true);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 0);
    }
}
