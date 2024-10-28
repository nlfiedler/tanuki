//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SearchParams {
    /// Tags that an asset should have. All should match.
    pub tags: Option<Vec<String>>,
    /// Locations of an asset. At least one must match.
    pub locations: Option<Vec<String>>,
    /// Date for filtering asset results. Only those assets whose canonical date
    /// occurs _on_ or _after_ this date will be returned.
    pub after: Option<DateTime<Utc>>,
    /// Date for filtering asset results. Only those assets whose canonical date
    /// occurs _before_ this date will be returned.
    pub before: Option<DateTime<Utc>>,
    /// Find assets whose filename (e.g. `img_3011.jpg`) matches the one given.
    pub filename: Option<String>,
    /// Find assets whose media type (e.g. `image/jpeg`) matches the one given.
    pub media_type: Option<String>,
    /// Field by which to sort the results.
    pub sort_field: Option<SortField>,
    /// Order by which to sort the results.
    pub sort_order: Option<SortOrder>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SearchMeta {
    /// The subset of results after applying pagination.
    pub results: Vec<SearchResult>,
    /// Total number of results matching the query.
    pub count: i32,
    /// Last possible page of available assets matching the query.
    pub last_page: i32,
}

/// A year for which there are `count` assets, converted from a `LabeledCount`.
#[derive(Clone, Deserialize, Serialize)]
pub struct Year {
    pub value: i32,
    pub count: usize,
}

impl From<LabeledCount> for Year {
    fn from(input: LabeledCount) -> Self {
        let value = i32::from_str_radix(&input.label, 10).unwrap_or(0);
        Self {
            value,
            count: input.count,
        }
    }
}

/// A 3-month period of the year, akin to seasons of anime.
#[derive(Copy, Clone, Deserialize, Eq, PartialEq, Serialize)]
pub enum Season {
    /// January through March
    Winter,
    /// April through June
    Spring,
    /// July through September
    Summer,
    /// October through December
    Fall,
}

impl fmt::Display for Season {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Season::Winter => write!(f, "Jan-Mar"),
            Season::Spring => write!(f, "Apr-Jun"),
            Season::Summer => write!(f, "Jul-Sep"),
            Season::Fall => write!(f, "Oct-Dec"),
        }
    }
}

/// Set of operations to transform the given list of assets.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct BulkEditParams {
    /// Identifiers of assets to be modified.
    pub assets: Vec<String>,
    /// Add or remove tags.
    pub tag_ops: Vec<TagOperation>,
    /// Modify the location.
    pub location_ops: Vec<LocationOperation>,
    /// Modify the user date.
    pub datetime_op: Option<DatetimeOperation>,
}
