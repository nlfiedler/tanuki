//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::{
    Asset, DatetimeOperation, Location, LocationField, LocationOperation, TagOperation,
};
use crate::domain::repositories::{RecordRepository, SearchRepository};
use anyhow::Error;
use std::cmp;
use std::fmt;

/// Use case to make changes to multiple assets at one time. The inputs include
/// the set of asset identifiers and the operations to be performed on those
/// assets.
pub struct EditAssets {
    records: Box<dyn RecordRepository>,
    cache: Box<dyn SearchRepository>,
}

impl EditAssets {
    pub fn new(records: Box<dyn RecordRepository>, cache: Box<dyn SearchRepository>) -> Self {
        Self { records, cache }
    }
}

impl super::UseCase<u64, Params> for EditAssets {
    fn call(&self, params: Params) -> Result<u64, Error> {
        let mut fixed_count: u64 = 0;
        for asset_id in params.assets.iter() {
            let mut asset = self.records.get_asset_by_id(&asset_id)?;
            if modifiy_asset(&mut asset, &params) {
                self.records.put_asset(&asset)?;
                fixed_count += 1;
            }
        }
        if fixed_count > 0 {
            self.cache.clear()?;
        }
        Ok(fixed_count)
    }
}

fn modifiy_asset(asset: &mut Asset, params: &Params) -> bool {
    let mut modified = false;
    for tag_op in params.tag_ops.iter() {
        match tag_op {
            TagOperation::Add(name) => {
                if !asset.tags.contains(name) {
                    asset.tags.push(name.to_owned());
                    modified = true;
                }
            }
            TagOperation::Remove(name) => {
                if asset.tags.contains(name) {
                    asset.tags.retain(|t| t != name);
                    modified = true;
                }
            }
        }
    }
    for loc_op in params.location_ops.iter() {
        match loc_op {
            LocationOperation::Set(field, value) => {
                if let Some(loc) = asset.location.as_mut() {
                    match field {
                        LocationField::Label => {
                            if let Some(label) = loc.label.as_ref() {
                                if label != value {
                                    loc.label = Some(value.to_owned());
                                    modified = true;
                                }
                            } else {
                                loc.label = Some(value.to_owned());
                                modified = true;
                            }
                        }
                        LocationField::City => {
                            if let Some(city) = loc.city.as_ref() {
                                if city != value {
                                    loc.city = Some(value.to_owned());
                                    modified = true;
                                }
                            } else {
                                loc.city = Some(value.to_owned());
                                modified = true;
                            }
                        }
                        LocationField::Region => {
                            if let Some(region) = loc.region.as_ref() {
                                if region != value {
                                    loc.region = Some(value.to_owned());
                                    modified = true;
                                }
                            } else {
                                loc.region = Some(value.to_owned());
                                modified = true;
                            }
                        }
                    }
                } else {
                    match field {
                        LocationField::Label => {
                            asset.location = Some(Location {
                                label: Some(value.to_owned()),
                                city: None,
                                region: None,
                            })
                        }
                        LocationField::City => {
                            asset.location = Some(Location {
                                label: None,
                                city: Some(value.to_owned()),
                                region: None,
                            })
                        }
                        LocationField::Region => {
                            asset.location = Some(Location {
                                label: None,
                                city: None,
                                region: Some(value.to_owned()),
                            })
                        }
                    }
                    modified = true;
                }
            }
            LocationOperation::Clear(field) => {
                if let Some(loc) = asset.location.as_mut() {
                    match field {
                        LocationField::Label => {
                            if loc.label.is_some() {
                                loc.label = None;
                                modified = true;
                            }
                        }
                        LocationField::City => {
                            if loc.city.is_some() {
                                loc.city = None;
                                modified = true;
                            }
                        }
                        LocationField::Region => {
                            if loc.region.is_some() {
                                loc.region = None;
                                modified = true;
                            }
                        }
                    }
                }
            }
        }
    }
    if let Some(date_op) = params.datetime_op.as_ref() {
        match date_op {
            DatetimeOperation::Set(datetime) => {
                let best_date = asset.best_date();
                if best_date != *datetime {
                    asset.user_date = Some(*datetime);
                    modified = true;
                }
            }
            DatetimeOperation::Add(days) => {
                let best_date = asset.best_date();
                let delta = chrono::TimeDelta::days(*days as i64);
                asset.user_date = best_date.checked_add_signed(delta);
                modified = true;
            }
            DatetimeOperation::Subtract(days) => {
                let best_date = asset.best_date();
                let delta = chrono::TimeDelta::days(*days as i64);
                asset.user_date = best_date.checked_sub_signed(delta);
                modified = true;
            }
            DatetimeOperation::Clear => {
                if asset.user_date.is_some() {
                    asset.user_date = None;
                    modified = true;
                }
            }
        }
    }
    modified
}

#[derive(Clone, Default)]
pub struct Params {
    /// Identifiers of the assets to be modified.
    pub assets: Vec<String>,
    /// Operations to perform on the tags.
    pub tag_ops: Vec<TagOperation>,
    /// Operations to perform on the location fields.
    pub location_ops: Vec<LocationOperation>,
    /// Optional date/time operation to perform.
    pub datetime_op: Option<DatetimeOperation>,
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Params({})", self.assets.len())
    }
}

impl cmp::PartialEq for Params {
    fn eq(&self, other: &Self) -> bool {
        self.assets == other.assets
    }
}

impl cmp::Eq for Params {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::Location;
    use chrono::prelude::*;

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
    fn test_modify_asset_add_tag() {
        let mut asset = Asset {
            key: "abc123".to_owned(),
            checksum: "deadbeef".to_owned(),
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
        let new_tag = String::from("bird");
        let params = Params {
            assets: vec!["abc123".into()],
            tag_ops: vec![TagOperation::Add(new_tag.clone())],
            location_ops: vec![],
            datetime_op: None,
        };
        let result = modifiy_asset(&mut asset, &params);
        assert!(result);
        assert_eq!(asset.tags.len(), 2);
        assert!(asset.tags.contains(&new_tag));

        // add a duplicate tag, should not change anything
        let result = modifiy_asset(&mut asset, &params);
        assert!(!result);
        assert_eq!(asset.tags.len(), 2);
    }

    #[test]
    fn test_modify_asset_remove_tag() {
        let mut asset = Asset {
            key: "abc123".to_owned(),
            checksum: "deadbeef".to_owned(),
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
        let old_tag = String::from("cow");
        let params = Params {
            assets: vec!["abc123".into()],
            tag_ops: vec![TagOperation::Remove(old_tag.clone())],
            location_ops: vec![],
            datetime_op: None,
        };
        let result = modifiy_asset(&mut asset, &params);
        assert!(result);
        assert_eq!(asset.tags.len(), 0);

        // repeat the same action again, nothing should change
        let result = modifiy_asset(&mut asset, &params);
        assert!(!result);
        assert_eq!(asset.tags.len(), 0);
    }

    #[test]
    fn test_modify_asset_replace_tag() {
        let mut asset = Asset {
            key: "abc123".to_owned(),
            checksum: "deadbeef".to_owned(),
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
        let new_tag = String::from("bovine");
        let old_tag = String::from("cow");
        let params = Params {
            assets: vec!["abc123".into()],
            tag_ops: vec![
                TagOperation::Remove(old_tag.clone()),
                TagOperation::Add(new_tag.clone()),
            ],
            location_ops: vec![],
            datetime_op: None,
        };
        let result = modifiy_asset(&mut asset, &params);
        assert!(result);
        assert_eq!(asset.tags.len(), 1);
        assert!(asset.tags.contains(&new_tag));

        // repeat the same action again, nothing should change
        let result = modifiy_asset(&mut asset, &params);
        assert!(!result);
        assert_eq!(asset.tags.len(), 1);
    }

    #[test]
    fn test_modify_asset_location_set_label() {
        let mut asset = Asset {
            key: "abc123".to_owned(),
            checksum: "deadbeef".to_owned(),
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
        let params = Params {
            assets: vec!["abc123".into()],
            tag_ops: vec![],
            location_ops: vec![LocationOperation::Set(
                LocationField::Label,
                "museum".into(),
            )],
            datetime_op: None,
        };
        let result = modifiy_asset(&mut asset, &params);
        assert!(result);
        assert_eq!(
            asset.location,
            Some(Location {
                label: Some("museum".into()),
                city: None,
                region: None,
            })
        );

        // repeat the same action again, nothing should change
        let result = modifiy_asset(&mut asset, &params);
        assert!(!result);
        assert_eq!(
            asset.location,
            Some(Location {
                label: Some("museum".into()),
                city: None,
                region: None,
            })
        );

        // change the label to something else
        let params = Params {
            assets: vec!["abc123".into()],
            tag_ops: vec![],
            location_ops: vec![LocationOperation::Set(LocationField::Label, "plaza".into())],
            datetime_op: None,
        };
        let result = modifiy_asset(&mut asset, &params);
        assert!(result);
        assert_eq!(
            asset.location,
            Some(Location {
                label: Some("plaza".into()),
                city: None,
                region: None,
            })
        );

        // clear the label field
        let params = Params {
            assets: vec!["abc123".into()],
            tag_ops: vec![],
            location_ops: vec![LocationOperation::Clear(LocationField::Label)],
            datetime_op: None,
        };
        let result = modifiy_asset(&mut asset, &params);
        assert!(result);
        assert_eq!(
            asset.location,
            Some(Location {
                label: None,
                city: None,
                region: None,
            })
        );

        // repeat again, nothing should change
        let result = modifiy_asset(&mut asset, &params);
        assert!(!result);
        assert_eq!(
            asset.location,
            Some(Location {
                label: None,
                city: None,
                region: None,
            })
        );
    }

    #[test]
    fn test_modify_asset_location_set_city() {
        let mut asset = Asset {
            key: "abc123".to_owned(),
            checksum: "deadbeef".to_owned(),
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
        let params = Params {
            assets: vec!["abc123".into()],
            tag_ops: vec![],
            location_ops: vec![LocationOperation::Set(
                LocationField::City,
                "Portland".into(),
            )],
            datetime_op: None,
        };
        let result = modifiy_asset(&mut asset, &params);
        assert!(result);
        assert_eq!(
            asset.location,
            Some(Location {
                label: None,
                city: Some("Portland".into()),
                region: None,
            })
        );

        // repeat the same action again, nothing should change
        let result = modifiy_asset(&mut asset, &params);
        assert!(!result);
        assert_eq!(
            asset.location,
            Some(Location {
                label: None,
                city: Some("Portland".into()),
                region: None,
            })
        );

        // change the city to something else
        let params = Params {
            assets: vec!["abc123".into()],
            tag_ops: vec![],
            location_ops: vec![LocationOperation::Set(
                LocationField::City,
                "Medford".into(),
            )],
            datetime_op: None,
        };
        let result = modifiy_asset(&mut asset, &params);
        assert!(result);
        assert_eq!(
            asset.location,
            Some(Location {
                label: None,
                city: Some("Medford".into()),
                region: None,
            })
        );

        // clear the city field
        let params = Params {
            assets: vec!["abc123".into()],
            tag_ops: vec![],
            location_ops: vec![LocationOperation::Clear(LocationField::City)],
            datetime_op: None,
        };
        let result = modifiy_asset(&mut asset, &params);
        assert!(result);
        assert_eq!(
            asset.location,
            Some(Location {
                label: None,
                city: None,
                region: None,
            })
        );

        // repeat again, nothing should change
        let result = modifiy_asset(&mut asset, &params);
        assert!(!result);
        assert_eq!(
            asset.location,
            Some(Location {
                label: None,
                city: None,
                region: None,
            })
        );
    }

    #[test]
    fn test_modify_asset_location_set_region() {
        let mut asset = Asset {
            key: "abc123".to_owned(),
            checksum: "deadbeef".to_owned(),
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
        let params = Params {
            assets: vec!["abc123".into()],
            tag_ops: vec![],
            location_ops: vec![LocationOperation::Set(
                LocationField::Region,
                "Oregon".into(),
            )],
            datetime_op: None,
        };
        let result = modifiy_asset(&mut asset, &params);
        assert!(result);
        assert_eq!(
            asset.location,
            Some(Location {
                label: None,
                city: None,
                region: Some("Oregon".into()),
            })
        );

        // repeat the same action again, nothing should change
        let result = modifiy_asset(&mut asset, &params);
        assert!(!result);
        assert_eq!(
            asset.location,
            Some(Location {
                label: None,
                city: None,
                region: Some("Oregon".into()),
            })
        );

        // change the region to something else
        let params = Params {
            assets: vec!["abc123".into()],
            tag_ops: vec![],
            location_ops: vec![LocationOperation::Set(
                LocationField::Region,
                "Washington".into(),
            )],
            datetime_op: None,
        };
        let result = modifiy_asset(&mut asset, &params);
        assert!(result);
        assert_eq!(
            asset.location,
            Some(Location {
                label: None,
                city: None,
                region: Some("Washington".into()),
            })
        );

        // clear the region field
        let params = Params {
            assets: vec!["abc123".into()],
            tag_ops: vec![],
            location_ops: vec![LocationOperation::Clear(LocationField::Region)],
            datetime_op: None,
        };
        let result = modifiy_asset(&mut asset, &params);
        assert!(result);
        assert_eq!(
            asset.location,
            Some(Location {
                label: None,
                city: None,
                region: None,
            })
        );

        // repeat again, nothing should change
        let result = modifiy_asset(&mut asset, &params);
        assert!(!result);
        assert_eq!(
            asset.location,
            Some(Location {
                label: None,
                city: None,
                region: None,
            })
        );
    }

    #[test]
    fn test_modify_asset_datetime() {
        let import_time = Utc::now();
        let mut asset = Asset {
            key: "abc123".to_owned(),
            checksum: "deadbeef".to_owned(),
            filename: "dcp_1069.jpg".to_owned(),
            byte_length: 80977,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cow".to_owned()],
            import_date: import_time,
            caption: None,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let expected = make_date_time(2018, 5, 31, 21, 10, 11);
        let params = Params {
            assets: vec!["abc123".into()],
            tag_ops: vec![],
            location_ops: vec![],
            datetime_op: Some(DatetimeOperation::Set(expected)),
        };
        let result = modifiy_asset(&mut asset, &params);
        assert!(result);
        assert_eq!(asset.best_date(), expected);

        // repeat the same action again, nothing should change
        let result = modifiy_asset(&mut asset, &params);
        assert!(!result);
        let best_date = asset.best_date();
        assert_eq!(best_date, expected);

        // add N days
        let params = Params {
            assets: vec!["abc123".into()],
            tag_ops: vec![],
            location_ops: vec![],
            datetime_op: Some(DatetimeOperation::Add(14)),
        };
        let result = modifiy_asset(&mut asset, &params);
        assert!(result);
        assert_eq!(asset.best_date(), make_date_time(2018, 6, 14, 21, 10, 11));

        // subtract N days
        let params = Params {
            assets: vec!["abc123".into()],
            tag_ops: vec![],
            location_ops: vec![],
            datetime_op: Some(DatetimeOperation::Subtract(14)),
        };
        let result = modifiy_asset(&mut asset, &params);
        assert!(result);
        assert_eq!(asset.best_date(), expected);

        // clear user date
        let params = Params {
            assets: vec!["abc123".into()],
            tag_ops: vec![],
            location_ops: vec![],
            datetime_op: Some(DatetimeOperation::Clear),
        };
        let result = modifiy_asset(&mut asset, &params);
        assert!(result);
        assert_eq!(asset.best_date(), import_time);

        // repeat the clear, nothing should change
        let params = Params {
            assets: vec!["abc123".into()],
            tag_ops: vec![],
            location_ops: vec![],
            datetime_op: Some(DatetimeOperation::Clear),
        };
        let result = modifiy_asset(&mut asset, &params);
        assert!(!result);
        assert_eq!(asset.best_date(), import_time);
    }
}
