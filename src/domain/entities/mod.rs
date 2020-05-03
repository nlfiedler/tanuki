//
// Copyright (c) 2020 Nathan Fiedler
//
use chrono::prelude::*;
use std::cmp;
use std::fmt;

/// Digital asset entity.
#[derive(Clone, Debug)]
pub struct Asset {
    /// The unique identifier of the asset.
    pub key: String,
    /// Hash digest of the asset contents.
    pub checksum: String,
    /// Original filename of the asset.
    pub filename: String,
    /// Size of the asset in bytes.
    pub byte_length: u64,
    /// Media type (formerly MIME type) of the asset.
    pub media_type: String,
    /// Set of user-assigned labels for the asset.
    pub tags: Vec<String>,
    /// Date when the asset was imported.
    pub import_date: DateTime<Utc>,
    /// Caption provided by the user.
    pub caption: Option<String>,
    /// User-defined location of the asset.
    pub location: Option<String>,
    /// Duration of (the video) asset in seconds.
    pub duration: Option<u32>,
    /// User-specified date of the asset.
    pub user_date: Option<DateTime<Utc>>,
    /// Date of the asset as extracted from metadata.
    pub original_date: Option<DateTime<Utc>>,
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Asset({}, {})", self.key, self.filename)
    }
}

impl cmp::PartialEq for Asset {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl cmp::Eq for Asset {}

/// A label and its associated count.
///
/// `LabeledCount` represents an attribute value and the number of occurrences
/// of that value in the data set. For instance, a location label and the number
/// of times that location occurs among the assets.
#[derive(Clone)]
pub struct LabeledCount {
    pub label: String,
    pub count: usize,
}

/// `SearchResult` is returned by data repository queries for assets matching a
/// given set of criteria.
#[derive(Clone, Debug)]
pub struct SearchResult {
    /// Asset identifier.
    pub asset_id: String,
    /// Original filename of the asset.
    pub filename: String,
    /// Media type (formerly MIME type) of the asset.
    pub media_type: String,
    /// User-defined location of the asset.
    pub location: Option<String>,
    /// Best date/time for the indexed asset.
    pub datetime: DateTime<Utc>,
}

impl SearchResult {
    /// Build a search result from the given asset.
    pub fn new(asset: &Asset) -> Self {
        let date = if let Some(ud) = asset.user_date.as_ref() {
            ud.to_owned()
        } else if let Some(od) = asset.original_date.as_ref() {
            od.to_owned()
        } else {
            asset.import_date
        };
        Self {
            asset_id: asset.key.clone(),
            filename: asset.filename.clone(),
            media_type: asset.media_type.clone(),
            location: asset.location.clone(),
            datetime: date,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_equality() {
        let asset1 = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cat".to_owned(), "dog".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: None,
            duration: None,
            user_date: None,
            original_date: None,
        };
        let asset2 = Asset {
            key: "abc123".to_owned(),
            checksum: "babecafe".to_owned(),
            filename: "img_4321.jpg".to_owned(),
            byte_length: 1_048_576,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kitten".to_owned(), "puppy".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: None,
            duration: None,
            user_date: None,
            original_date: None,
        };
        assert!(asset1 == asset2);
        assert!(asset2 == asset1);
        let asset3 = Asset {
            key: "xyz789".to_owned(),
            checksum: "babecafe".to_owned(),
            filename: "img_4321.jpg".to_owned(),
            byte_length: 1_048_576,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kitten".to_owned(), "puppy".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: None,
            duration: None,
            user_date: None,
            original_date: None,
        };
        assert!(asset1 != asset3);
        assert!(asset2 != asset3);
    }

    #[test]
    fn test_asset_stringify() {
        let asset1 = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cat".to_owned(), "dog".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: None,
            duration: None,
            user_date: None,
            original_date: None,
        };
        let actual = asset1.to_string();
        assert_eq!(actual, "Asset(abc123, img_1234.jpg)");
    }

    #[test]
    fn test_search_result_new_user_date() {
        // arrange
        let asset = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cat".to_owned(), "dog".to_owned()],
            import_date: Utc.ymd(2017, 4, 28).and_hms(11, 12, 59),
            caption: None,
            location: None,
            duration: None,
            user_date: Some(Utc.ymd(2018, 5, 31).and_hms(21, 10, 11)),
            original_date: Some(Utc.ymd(2016, 8, 30).and_hms(12, 10, 30)),
        };
        // act
        let result = SearchResult::new(&asset);
        // assert
        assert_eq!(result.datetime.year(), 2018);
    }

    #[test]
    fn test_search_result_new_original_date() {
        // arrange
        let asset = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cat".to_owned(), "dog".to_owned()],
            import_date: Utc.ymd(2017, 4, 28).and_hms(11, 12, 59),
            caption: None,
            location: None,
            duration: None,
            user_date: None,
            original_date: Some(Utc.ymd(2016, 8, 30).and_hms(12, 10, 30)),
        };
        // act
        let result = SearchResult::new(&asset);
        // assert
        assert_eq!(result.datetime.year(), 2016);
    }

    #[test]
    fn test_search_result_new_import_date() {
        // arrange
        let asset = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cat".to_owned(), "dog".to_owned()],
            import_date: Utc.ymd(2017, 4, 28).and_hms(11, 12, 59),
            caption: None,
            location: None,
            duration: None,
            user_date: None,
            original_date: None,
        };
        // act
        let result = SearchResult::new(&asset);
        // assert
        assert_eq!(result.datetime.year(), 2017);
    }
}
