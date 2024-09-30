//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::{LabeledCount, SearchResult, SortField, SortOrder};
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

/// Return the optional value bounded by the given range, or the default value
/// if `value` is `None`.
fn bounded_int_value(value: Option<i32>, default: i32, minimum: i32, maximum: i32) -> i32 {
    if let Some(v) = value {
        std::cmp::min(std::cmp::max(v, minimum), maximum)
    } else {
        default
    }
}

/// Truncate the given vector to yield the desired portion.
///
/// If offset is None, it defaults to 0, while count defaults to 10. Offset is
/// bound between zero and the length of the input vector. Count is bound by 1
/// and 250.
pub fn paginate_vector<T>(input: &mut Vec<T>, offset: Option<i32>, count: Option<i32>) -> Vec<T> {
    let total_count = input.len() as i32;
    let count = bounded_int_value(count, 10, 1, 250) as usize;
    let offset = bounded_int_value(offset, 0, 0, total_count) as usize;
    let mut results = input.split_off(offset);
    results.truncate(count);
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded_int_value() {
        assert_eq!(10, bounded_int_value(None, 10, 1, 250));
        assert_eq!(15, bounded_int_value(Some(15), 10, 1, 250));
        assert_eq!(1, bounded_int_value(Some(-8), 10, 1, 250));
        assert_eq!(250, bounded_int_value(Some(1000), 10, 1, 250));
    }

    #[test]
    fn test_paginate_vector() {
        // sensible "first" page
        let mut input: Vec<u32> = Vec::new();
        for v in 0..102 {
            input.push(v);
        }
        let actual = paginate_vector(&mut input, Some(0), Some(10));
        assert_eq!(actual.len(), 10);
        assert_eq!(actual[0], 0);
        assert_eq!(actual[9], 9);

        // page somewhere in the middle
        let mut input: Vec<u32> = Vec::new();
        for v in 0..102 {
            input.push(v);
        }
        let actual = paginate_vector(&mut input, Some(40), Some(20));
        assert_eq!(actual.len(), 20);
        assert_eq!(actual[0], 40);
        assert_eq!(actual[19], 59);

        // last page with over extension
        let mut input: Vec<u32> = Vec::new();
        for v in 0..102 {
            input.push(v);
        }
        let actual = paginate_vector(&mut input, Some(90), Some(100));
        assert_eq!(actual.len(), 12);
        assert_eq!(actual[0], 90);
        assert_eq!(actual[11], 101);
    }
}
