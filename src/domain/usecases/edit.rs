//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::{Asset, Location};
use crate::domain::repositories::RecordRepository;
use anyhow::Error;
use chrono::prelude::*;
use log::info;
use std::cmp;
use std::fmt;

/// Use case to find assets that match certain criteria and make one or more
/// changes as appropriate. Like a search and replace with more options.
pub struct EditAssets {
    records: Box<dyn RecordRepository>,
}

impl EditAssets {
    pub fn new(records: Box<dyn RecordRepository>) -> Self {
        Self { records }
    }
}

impl super::UseCase<u64, Params> for EditAssets {
    fn call(&self, params: Params) -> Result<u64, Error> {
        let mut fixed_count: u64 = 0;
        if params.filter.is_empty() {
            info!("empty filter, nothing to do");
        } else {
            let all_assets = self.records.all_assets()?;
            for asset_id in all_assets {
                info!("evaluating asset {}", asset_id);
                let mut asset = self.records.get_asset(&asset_id)?;
                if asset_matches(&asset, &params.filter) {
                    if modifiy_asset(&mut asset, &params) {
                        self.records.put_asset(&asset)?;
                        info!("modified asset {}", asset_id);
                        fixed_count += 1;
                    }
                }
            }
            info!("scan complete");
        }
        Ok(fixed_count)
    }
}

fn asset_matches(asset: &Asset, filter: &Filter) -> bool {
    if let Some(media_type) = filter.media_type.as_ref() {
        if &asset.media_type != media_type {
            return false;
        }
    }
    if !filter_by_date_range(asset, filter) {
        return false;
    }
    if !location_matches(asset.location.as_ref(), filter.location.as_ref()) {
        return false;
    }
    tags_match(&asset.tags, &filter.tags)
}

// Determine if the asset's best date/time falls within the range given by the
// filter, either before, after, or between the before and after datetimes.
fn filter_by_date_range(asset: &Asset, filter: &Filter) -> bool {
    let best_date = asset.best_date();
    if filter.after_date.is_some() && filter.before_date.is_some() {
        let a = filter.after_date.unwrap();
        let b = filter.before_date.unwrap();
        best_date > a && best_date < b
    } else if filter.after_date.is_some() {
        let a = filter.after_date.unwrap();
        best_date > a
    } else if filter.before_date.is_some() {
        let b = filter.before_date.unwrap();
        best_date < b
    } else {
        true
    }
}

fn location_part_matches(asset: Option<&String>, filter: Option<&String>) -> bool {
    if let Some(fstr) = filter {
        if fstr.is_empty() && asset.is_none() {
            return true;
        }
    } else {
        return true;
    }
    if asset != filter {
        return false;
    }
    true
}

fn all_blank_location(filter: &Location) -> bool {
    filter.label.as_ref().filter(|s| !s.is_empty()).is_none()
        && filter.city.as_ref().filter(|s| !s.is_empty()).is_none()
        && filter.region.as_ref().filter(|s| !s.is_empty()).is_none()
}

fn location_matches(asset: Option<&Location>, filter: Option<&Location>) -> bool {
    if let Some(floc) = filter {
        if let Some(aloc) = asset {
            if !location_part_matches(aloc.label.as_ref(), floc.label.as_ref()) {
                return false;
            }
            if !location_part_matches(aloc.city.as_ref(), floc.city.as_ref()) {
                return false;
            }
            if !location_part_matches(aloc.region.as_ref(), floc.region.as_ref()) {
                return false;
            }
        } else if !all_blank_location(floc) {
            return false;
        }
    }
    true
}

fn tags_match(asset: &Vec<String>, filter: &Vec<String>) -> bool {
    // if filter tags list is longer, then match is not possible
    if asset.len() < filter.len() {
        return false;
    }
    // the list of tags should be short, n^2 search is fine
    for ftag in filter.iter() {
        if asset.iter().any(|t| t == ftag) == false {
            return false;
        }
    }
    true
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

/// Filter to select assets for editing.
#[derive(Clone, Default)]
pub struct Filter {
    /// Asset must have all of these tags.
    pub tags: Vec<String>,
    /// Asset location must match defined fields of location. If any field is an
    /// empty string, then the corresponding asset field must be undefined.
    pub location: Option<Location>,
    /// Asset "best" date must be before this date.
    pub before_date: Option<DateTime<Utc>>,
    /// Asset "best" date must be after this date.
    pub after_date: Option<DateTime<Utc>>,
    /// Asset media type must match this value.
    pub media_type: Option<String>,
}

impl Filter {
    fn is_empty(&self) -> bool {
        self.tags.is_empty()
            && self.location.is_none()
            && self.before_date.is_none()
            && self.after_date.is_none()
            && self.media_type.is_none()
    }

    /// Add the given tag to the list of tags for this filter.
    pub fn tag<T: Into<String>>(mut self, name: T) -> Self {
        self.tags.push(name.into());
        self
    }
}

/// Action to perform on the asset tags.
#[derive(Clone)]
pub enum TagOperation {
    /// Add a tag with the given value.
    Add(String),
    /// Remove the tag that matches the given value.
    Remove(String),
}

/// Identify the field of the location.
#[derive(Clone)]
pub enum LocationField {
    Label,
    City,
    Region,
}

/// Action to perform on the asset location.
#[derive(Clone)]
pub enum LocationOperation {
    /// Set the value for the corresponding field.
    Set(LocationField, String),
    /// Clear the corresponding field.
    Clear(LocationField),
}

/// Set, clear, add, or subtract from the asset date.
#[derive(Clone)]
pub enum DatetimeOperation {
    /// Set the "user" date to the value given.
    Set(DateTime<Utc>),
    /// Add the given number of days to the best date, save as "user" date.
    Add(u16),
    /// Subtract the given number of days from the best date, save as "user" date.
    Subtract(u16),
    /// Clear the "user" date field.
    Clear,
}

#[derive(Clone, Default)]
pub struct Params {
    /// Criteria for finding assets to be modified.
    pub filter: Filter,
    /// Operations to perform on the tags.
    pub tag_ops: Vec<TagOperation>,
    /// Operations to perform on the location fields.
    pub location_ops: Vec<LocationOperation>,
    /// Optional date/time operation to perform.
    pub datetime_op: Option<DatetimeOperation>,
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Params(tags: {})", self.filter.tags.len())
    }
}

impl cmp::PartialEq for Params {
    fn eq(&self, other: &Self) -> bool {
        self.filter.tags == other.filter.tags
    }
}

impl cmp::Eq for Params {}

#[cfg(test)]
mod tests {
    use super::super::UseCase;
    use super::*;
    use crate::domain::entities::Location;
    use crate::domain::repositories::MockRecordRepository;
    use mockall::predicate::*;

    #[test]
    fn test_all_blank_location() {
        let loc = Location {
            label: None,
            city: None,
            region: None,
        };
        assert!(all_blank_location(&loc));

        let loc = Location {
            label: Some("".into()),
            city: None,
            region: None,
        };
        assert!(all_blank_location(&loc));

        let loc = Location {
            label: None,
            city: Some("".into()),
            region: None,
        };
        assert!(all_blank_location(&loc));

        let loc = Location {
            label: None,
            city: None,
            region: Some("".into()),
        };
        assert!(all_blank_location(&loc));

        let loc = Location {
            label: Some("foo".into()),
            city: None,
            region: None,
        };
        assert!(!all_blank_location(&loc));

        let loc = Location {
            label: None,
            city: Some("foo".into()),
            region: None,
        };
        assert!(!all_blank_location(&loc));

        let loc = Location {
            label: None,
            city: None,
            region: Some("foo".into()),
        };
        assert!(!all_blank_location(&loc));
    }

    #[test]
    fn test_location_part_matches() {
        let asset: Option<String> = Some("foo".into());
        let filter: Option<String> = Some("foo".into());
        assert!(location_part_matches(asset.as_ref(), filter.as_ref()));

        let asset: Option<String> = Some("foo".into());
        let filter: Option<String> = None;
        assert!(location_part_matches(asset.as_ref(), filter.as_ref()));

        let asset: Option<String> = None;
        let filter: Option<String> = None;
        assert!(location_part_matches(asset.as_ref(), filter.as_ref()));

        let asset: Option<String> = None;
        let filter: Option<String> = Some("".into());
        assert!(location_part_matches(asset.as_ref(), filter.as_ref()));

        let asset: Option<String> = Some("foo".into());
        let filter: Option<String> = Some("".into());
        assert!(!location_part_matches(asset.as_ref(), filter.as_ref()));

        let asset: Option<String> = Some("foo".into());
        let filter: Option<String> = Some("bar".into());
        assert!(!location_part_matches(asset.as_ref(), filter.as_ref()));
    }

    #[test]
    fn test_location_matches_label() {
        let mut asset = Some(Location {
            label: Some("home".into()),
            city: None,
            region: None,
        });
        let mut filter = Some(Location {
            label: Some("".into()),
            city: None,
            region: None,
        });
        let actual = location_matches(asset.as_ref(), filter.as_ref());
        assert!(!actual, "label should be none");

        filter = Some(Location {
            label: Some("foobar".into()),
            city: None,
            region: None,
        });
        let actual = location_matches(asset.as_ref(), filter.as_ref());
        assert!(!actual, "label should not match");

        filter = Some(Location {
            label: Some("home".into()),
            city: None,
            region: None,
        });
        let actual = location_matches(asset.as_ref(), filter.as_ref());
        assert!(actual, "label should match");

        asset = None;
        let actual = location_matches(asset.as_ref(), filter.as_ref());
        assert!(!actual, "label should be some");

        filter = Some(Location {
            label: Some("".into()),
            city: None,
            region: None,
        });
        let actual = location_matches(asset.as_ref(), filter.as_ref());
        assert!(actual, "label should be none");

        filter = Some(Location {
            label: None,
            city: None,
            region: None,
        });
        let actual = location_matches(asset.as_ref(), filter.as_ref());
        assert!(actual, "label should be none");
    }

    #[test]
    fn test_location_matches_city() {
        let mut asset = Some(Location {
            label: None,
            city: Some("home".into()),
            region: None,
        });
        let mut filter = Some(Location {
            label: None,
            city: Some("".into()),
            region: None,
        });
        let actual = location_matches(asset.as_ref(), filter.as_ref());
        assert!(!actual, "city should be none");

        filter = Some(Location {
            label: None,
            city: Some("foobar".into()),
            region: None,
        });
        let actual = location_matches(asset.as_ref(), filter.as_ref());
        assert!(!actual, "city should not match");

        filter = Some(Location {
            label: None,
            city: Some("home".into()),
            region: None,
        });
        let actual = location_matches(asset.as_ref(), filter.as_ref());
        assert!(actual, "city should match");

        asset = None;
        let actual = location_matches(asset.as_ref(), filter.as_ref());
        assert!(!actual, "city should be some");

        filter = Some(Location {
            label: None,
            city: Some("".into()),
            region: None,
        });
        let actual = location_matches(asset.as_ref(), filter.as_ref());
        assert!(actual, "city should be none");

        filter = Some(Location {
            label: None,
            city: None,
            region: None,
        });
        let actual = location_matches(asset.as_ref(), filter.as_ref());
        assert!(actual, "city should be none");
    }

    #[test]
    fn test_location_matches_region() {
        let mut asset = Some(Location {
            label: None,
            city: None,
            region: Some("home".into()),
        });
        let mut filter = Some(Location {
            label: None,
            city: None,
            region: Some("".into()),
        });
        let actual = location_matches(asset.as_ref(), filter.as_ref());
        assert!(!actual, "region should be none");

        filter = Some(Location {
            label: None,
            city: None,
            region: Some("foobar".into()),
        });
        let actual = location_matches(asset.as_ref(), filter.as_ref());
        assert!(!actual, "region should not match");

        filter = Some(Location {
            label: None,
            city: None,
            region: Some("home".into()),
        });
        let actual = location_matches(asset.as_ref(), filter.as_ref());
        assert!(actual, "region should match");

        asset = None;
        let actual = location_matches(asset.as_ref(), filter.as_ref());
        assert!(!actual, "region should be some");

        filter = Some(Location {
            label: None,
            city: None,
            region: Some("".into()),
        });
        let actual = location_matches(asset.as_ref(), filter.as_ref());
        assert!(actual, "region should be none");

        filter = Some(Location {
            label: None,
            city: None,
            region: None,
        });
        let actual = location_matches(asset.as_ref(), filter.as_ref());
        assert!(actual, "region should be none");
    }

    #[test]
    fn test_location_matches() {
        let asset = Some(Location {
            label: Some("home".into()),
            city: Some("Portland".into()),
            region: Some("Oregon".into()),
        });
        let filter = None;
        assert!(location_matches(asset.as_ref(), filter.as_ref()));

        let asset = None;
        let filter = Some(Location {
            label: None,
            city: Some("Portland".into()),
            region: Some("Oregon".into()),
        });
        assert!(!location_matches(asset.as_ref(), filter.as_ref()));

        let asset = Some(Location {
            label: Some("home".into()),
            city: Some("Portland".into()),
            region: Some("Oregon".into()),
        });
        let filter = Some(Location {
            label: None,
            city: None,
            region: Some("Oregon".into()),
        });
        assert!(location_matches(asset.as_ref(), filter.as_ref()));

        let asset = Some(Location {
            label: Some("home".into()),
            city: Some("Portland".into()),
            region: Some("Oregon".into()),
        });
        let filter = Some(Location {
            label: Some("home".into()),
            city: None,
            region: Some("Oregon".into()),
        });
        assert!(location_matches(asset.as_ref(), filter.as_ref()));

        let asset = Some(Location {
            label: Some("home".into()),
            city: Some("Portland".into()),
            region: Some("Oregon".into()),
        });
        let filter = Some(Location {
            label: None,
            city: Some("Portland".into()),
            region: Some("Oregon".into()),
        });
        assert!(location_matches(asset.as_ref(), filter.as_ref()));

        let asset = Some(Location {
            label: Some("home".into()),
            city: Some("Portland".into()),
            region: Some("Oregon".into()),
        });
        let filter = Some(Location {
            label: None,
            city: Some("".into()),
            region: Some("Oregon".into()),
        });
        assert!(!location_matches(asset.as_ref(), filter.as_ref()));

        let asset = Some(Location {
            label: Some("home".into()),
            city: Some("Portland".into()),
            region: Some("Oregon".into()),
        });
        let filter = Some(Location {
            label: None,
            city: Some("Eugene".into()),
            region: Some("Oregon".into()),
        });
        assert!(!location_matches(asset.as_ref(), filter.as_ref()));
    }

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
    fn test_filter_by_date_range() {
        let asset = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "dcp_1069.jpg".to_owned(),
            byte_length: 80977,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cow".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: None,
            user_date: Some(make_date_time(2018, 5, 31, 21, 10, 11)),
            original_date: None,
            dimensions: None,
        };
        let mut filter: Filter = Default::default();
        filter.before_date = Some(make_date_time(2018, 6, 1, 12, 10, 11));
        assert!(filter_by_date_range(&asset, &filter));

        filter.before_date = Some(make_date_time(2017, 6, 1, 12, 10, 11));
        assert!(!filter_by_date_range(&asset, &filter));

        filter.before_date = None;
        filter.after_date = Some(make_date_time(2017, 6, 1, 12, 10, 11));
        assert!(filter_by_date_range(&asset, &filter));

        filter.before_date = Some(make_date_time(2019, 6, 1, 12, 10, 11));
        filter.after_date = Some(make_date_time(2017, 6, 1, 12, 10, 11));
        assert!(filter_by_date_range(&asset, &filter));
    }

    #[test]
    fn test_tags_match() {
        let asset: Vec<String> = vec![];
        let filter: Vec<String> = vec![];
        assert!(tags_match(&asset, &filter));

        let asset: Vec<String> = vec![];
        let filter: Vec<String> = vec!["dog".into()];
        assert!(!tags_match(&asset, &filter));

        let asset: Vec<String> = vec!["dog".into()];
        let filter: Vec<String> = vec!["dog".into()];
        assert!(tags_match(&asset, &filter));

        let asset: Vec<String> = vec!["cat".into()];
        let filter: Vec<String> = vec!["dog".into()];
        assert!(!tags_match(&asset, &filter));

        let asset: Vec<String> = vec!["cat".into(), "dog".into()];
        let filter: Vec<String> = vec!["dog".into(), "cat".into()];
        assert!(tags_match(&asset, &filter));

        let asset: Vec<String> = vec!["bird".into(), "dog".into()];
        let filter: Vec<String> = vec!["dog".into(), "cat".into()];
        assert!(!tags_match(&asset, &filter));

        let asset: Vec<String> = vec!["bird".into(), "cat".into(), "dog".into()];
        let filter: Vec<String> = vec!["dog".into(), "cat".into(), "bird".into()];
        assert!(tags_match(&asset, &filter));

        let asset: Vec<String> = vec!["bird".into(), "dog".into(), "zebra".into()];
        let filter: Vec<String> = vec![];
        assert!(tags_match(&asset, &filter));

        let asset: Vec<String> = vec!["bird".into(), "dog".into(), "zebra".into()];
        let filter: Vec<String> = vec!["dog".into()];
        assert!(tags_match(&asset, &filter));

        let asset: Vec<String> = vec!["bird".into(), "dog".into(), "cat".into()];
        let filter: Vec<String> = vec!["cat".into(), "dog".into()];
        assert!(tags_match(&asset, &filter));

        let asset: Vec<String> = vec!["bird".into(), "dog".into(), "zebra".into()];
        let filter: Vec<String> = vec!["dog".into(), "cat".into()];
        assert!(!tags_match(&asset, &filter));
    }

    #[test]
    fn test_asset_matches_blank_filter() {
        let asset = Asset {
            key: "abc123".to_owned(),
            checksum: "deadbeef".to_owned(),
            filename: "dcp_1069.jpg".to_owned(),
            byte_length: 80977,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cow".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: Some(Location {
                label: None,
                city: Some("foo".into()),
                region: Some("bar".into()),
            }),
            user_date: Some(make_date_time(2018, 5, 31, 21, 10, 11)),
            original_date: None,
            dimensions: None,
        };
        let filter: Filter = Default::default();
        assert!(asset_matches(&asset, &filter));
    }

    #[test]
    fn test_asset_matches_media_type() {
        let asset = Asset {
            key: "abc123".to_owned(),
            checksum: "deadbeef".to_owned(),
            filename: "dcp_1069.jpg".to_owned(),
            byte_length: 80977,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cow".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: Some(Location {
                label: None,
                city: Some("foo".into()),
                region: Some("bar".into()),
            }),
            user_date: Some(make_date_time(2018, 5, 31, 21, 10, 11)),
            original_date: None,
            dimensions: None,
        };
        let mut filter: Filter = Default::default();
        filter.media_type = Some("video/mp4".into());
        assert!(!asset_matches(&asset, &filter));
    }

    #[test]
    fn test_asset_matches_before_date() {
        let asset = Asset {
            key: "abc123".to_owned(),
            checksum: "deadbeef".to_owned(),
            filename: "dcp_1069.jpg".to_owned(),
            byte_length: 80977,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cow".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: Some(Location {
                label: None,
                city: Some("foo".into()),
                region: Some("bar".into()),
            }),
            user_date: Some(make_date_time(2018, 5, 31, 21, 10, 11)),
            original_date: None,
            dimensions: None,
        };
        let mut filter: Filter = Default::default();
        filter.before_date = Some(make_date_time(2017, 5, 31, 21, 10, 11));
        assert!(!asset_matches(&asset, &filter));
    }

    #[test]
    fn test_asset_matches_location() {
        let asset = Asset {
            key: "abc123".to_owned(),
            checksum: "deadbeef".to_owned(),
            filename: "dcp_1069.jpg".to_owned(),
            byte_length: 80977,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cow".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: Some(Location {
                label: None,
                city: Some("foo".into()),
                region: Some("bar".into()),
            }),
            user_date: Some(make_date_time(2018, 5, 31, 21, 10, 11)),
            original_date: None,
            dimensions: None,
        };
        let mut filter: Filter = Default::default();
        filter.location = Some(Location {
            label: None,
            city: Some("quux".into()),
            region: None,
        });
        assert!(!asset_matches(&asset, &filter));
    }

    #[test]
    fn test_asset_matches_tags() {
        let asset = Asset {
            key: "abc123".to_owned(),
            checksum: "deadbeef".to_owned(),
            filename: "dcp_1069.jpg".to_owned(),
            byte_length: 80977,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cow".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: Some(Location {
                label: None,
                city: Some("foo".into()),
                region: Some("bar".into()),
            }),
            user_date: Some(make_date_time(2018, 5, 31, 21, 10, 11)),
            original_date: None,
            dimensions: None,
        };
        let mut filter: Filter = Default::default();
        filter.tags = vec!["bird".into()];
        assert!(!asset_matches(&asset, &filter));
    }

    #[test]
    fn test_asset_matches_all_filters() {
        let asset = Asset {
            key: "abc123".to_owned(),
            checksum: "deadbeef".to_owned(),
            filename: "dcp_1069.jpg".to_owned(),
            byte_length: 80977,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cow".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: Some(Location {
                label: None,
                city: Some("foo".into()),
                region: Some("bar".into()),
            }),
            user_date: Some(make_date_time(2018, 5, 31, 21, 10, 11)),
            original_date: None,
            dimensions: None,
        };
        let mut filter: Filter = Default::default();
        filter.after_date = Some(make_date_time(2017, 5, 31, 21, 10, 11));
        filter.before_date = Some(make_date_time(2019, 5, 31, 21, 10, 11));
        filter.location = Some(Location {
            label: None,
            city: Some("foo".into()),
            region: Some("bar".into()),
        });
        filter.media_type = Some("image/jpeg".into());
        filter.tags = vec!["cow".into()];
        assert!(asset_matches(&asset, &filter));
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
            filter: Default::default(),
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
            filter: Default::default(),
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
            filter: Default::default(),
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
            filter: Default::default(),
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
            filter: Default::default(),
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
            filter: Default::default(),
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
            filter: Default::default(),
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
            filter: Default::default(),
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
            filter: Default::default(),
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
            filter: Default::default(),
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
            filter: Default::default(),
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
            filter: Default::default(),
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
            filter: Default::default(),
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
            filter: Default::default(),
            tag_ops: vec![],
            location_ops: vec![],
            datetime_op: Some(DatetimeOperation::Add(14)),
        };
        let result = modifiy_asset(&mut asset, &params);
        assert!(result);
        assert_eq!(asset.best_date(), make_date_time(2018, 6, 14, 21, 10, 11));

        // subtract N days
        let params = Params {
            filter: Default::default(),
            tag_ops: vec![],
            location_ops: vec![],
            datetime_op: Some(DatetimeOperation::Subtract(14)),
        };
        let result = modifiy_asset(&mut asset, &params);
        assert!(result);
        assert_eq!(asset.best_date(), expected);

        // clear user date
        let params = Params {
            filter: Default::default(),
            tag_ops: vec![],
            location_ops: vec![],
            datetime_op: Some(DatetimeOperation::Clear),
        };
        let result = modifiy_asset(&mut asset, &params);
        assert!(result);
        assert_eq!(asset.best_date(), import_time);

        // repeat the clear, nothing should change
        let params = Params {
            filter: Default::default(),
            tag_ops: vec![],
            location_ops: vec![],
            datetime_op: Some(DatetimeOperation::Clear),
        };
        let result = modifiy_asset(&mut asset, &params);
        assert!(!result);
        assert_eq!(asset.best_date(), import_time);
    }

    #[test]
    fn test_edit_assets_empty_filter() {
        // arrange
        let records = MockRecordRepository::new();
        // act
        let usecase = EditAssets::new(Box::new(records));
        let params = Params {
            filter: Default::default(),
            tag_ops: vec![],
            location_ops: vec![],
            datetime_op: Some(DatetimeOperation::Clear),
        };
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_edit_assets_no_matches() {
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
        let usecase = EditAssets::new(Box::new(records));
        let mut filter: Filter = Default::default();
        filter = filter.tag("alien");
        let params = Params {
            filter,
            tag_ops: vec![TagOperation::Add("birds".into())],
            location_ops: vec![],
            datetime_op: None,
        };
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_edit_assets_matches_tag() {
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
        let angel = String::from("angel");
        records
            .expect_put_asset()
            .withf(move |asset| asset.key == asset1_id && asset.tags.contains(&angel))
            .returning(|_| Ok(()));
        // act
        let usecase = EditAssets::new(Box::new(records));
        let mut filter: Filter = Default::default();
        filter = filter.tag("coaster");
        let params = Params {
            filter,
            tag_ops: vec![TagOperation::Add("angel".into())],
            location_ops: vec![],
            datetime_op: None,
        };
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 1);
    }
}
