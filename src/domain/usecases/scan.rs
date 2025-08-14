//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::{SearchResult, SortField, SortOrder};
use crate::domain::repositories::{RecordRepository, SearchRepository};
use anyhow::Error;
use log::info;
use query::Constraint;
use std::cmp;
use std::fmt;

/// Use case to scan all assets in the database, matching against multiple
/// criteria with optional boolean operators and grouping.
pub struct ScanAssets {
    repo: Box<dyn RecordRepository>,
    cache: Box<dyn SearchRepository>,
}

impl ScanAssets {
    pub fn new(repo: Box<dyn RecordRepository>, cache: Box<dyn SearchRepository>) -> Self {
        Self { repo, cache }
    }
}

impl super::UseCase<Vec<SearchResult>, Params> for ScanAssets {
    fn call(&self, params: Params) -> Result<Vec<SearchResult>, Error> {
        use crate::domain::usecases::scan::query::Predicate;
        let cons = parser::parse_query(&params.query)?;
        let mut results: Vec<SearchResult> = vec![];
        if matches!(cons, Constraint::Empty) {
            return Ok(results);
        }

        if let Some(cached) = self.cache.get(&params.query)? {
            results = cached;
        } else {
            // use a cursor to iterate all of the assets in batches
            let mut scan_count: usize = 0;
            let mut cursor: Option<String> = None;
            loop {
                let mut batch = self.repo.fetch_assets(cursor, 1024)?;
                scan_count += batch.assets.len();
                for asset in batch.assets.into_iter() {
                    if cons.matches(&asset) {
                        results.push(SearchResult::new(&asset));
                    }
                }
                // stop when all records have been scanned
                cursor = batch.cursor.take();
                if cursor.is_none() {
                    break;
                }
            }
            info!(
                "scanned {} total assets, {} matching",
                scan_count,
                results.len()
            );
            self.cache.put(params.query, results.clone())?;
        }
        super::sort_results(&mut results, params.sort_field, params.sort_order);
        Ok(results)
    }
}

#[derive(Clone, Default)]
pub struct Params {
    pub query: String,
    pub sort_field: Option<SortField>,
    pub sort_order: Option<SortOrder>,
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Params(query: {})", self.query)
    }
}

impl cmp::PartialEq for Params {
    fn eq(&self, other: &Self) -> bool {
        self.query == other.query
    }
}

impl cmp::Eq for Params {}

#[cfg(test)]
mod test {
    use super::super::UseCase;
    use super::*;
    use crate::domain::entities::{Asset, Location};
    use crate::domain::repositories::{FetchedAssets, MockRecordRepository, MockSearchRepository};
    use chrono::prelude::*;

    #[test]
    fn test_scan_empty_query() {
        // arrange
        let mut mock = MockRecordRepository::new();
        mock.expect_fetch_assets().never();
        let mut cache = MockSearchRepository::new();
        cache.expect_get().never();
        cache.expect_put().never();
        // act
        let usecase = ScanAssets::new(Box::new(mock), Box::new(cache));
        let params = Params {
            query: "    ".into(),
            sort_field: None,
            sort_order: None,
        };
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_scan_zero_assets() {
        // arrange
        let mut mock = MockRecordRepository::new();
        mock.expect_fetch_assets().returning(|_, _| {
            Ok(FetchedAssets {
                assets: vec![],
                cursor: None,
            })
        });
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = ScanAssets::new(Box::new(mock), Box::new(cache));
        let params = Params {
            query: "tag:rainbows".into(),
            sort_field: None,
            sort_order: None,
        };
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_scan_no_results() {
        // arrange
        let mut mock = MockRecordRepository::new();
        mock.expect_fetch_assets()
            .withf(|c, _| c.is_none())
            .returning(move |_, _| {
                Ok(FetchedAssets {
                    assets: vec![Asset {
                        key: "abc123".to_owned(),
                        checksum: "cafebabe".to_owned(),
                        filename: "img_1234.jpg".to_owned(),
                        byte_length: 1024,
                        media_type: "image/jpeg".to_owned(),
                        tags: vec!["cat".to_owned(), "dog".to_owned()],
                        import_date: Utc::now(),
                        caption: None,
                        location: None,
                        user_date: None,
                        original_date: None,
                        dimensions: None,
                    }],
                    cursor: None,
                })
            });
        mock.expect_fetch_assets()
            .withf(|c, _| c.is_some())
            .returning(|_, _| {
                Ok(FetchedAssets {
                    assets: vec![],
                    cursor: None,
                })
            });
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = ScanAssets::new(Box::new(mock), Box::new(cache));
        let params = Params {
            query: "tag:horses".into(),
            sort_field: None,
            sort_order: None,
        };
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_scan_one_result() {
        // arrange
        let mut mock = MockRecordRepository::new();
        mock.expect_fetch_assets()
            .withf(|c, _| c.is_none())
            .returning(move |_, _| {
                Ok(FetchedAssets {
                    assets: vec![
                        Asset {
                            key: "abc123".to_owned(),
                            checksum: "cafebabe".to_owned(),
                            filename: "img_1234.jpg".to_owned(),
                            byte_length: 1024,
                            media_type: "image/jpeg".to_owned(),
                            tags: vec!["cat".to_owned(), "dog".to_owned()],
                            import_date: Utc::now(),
                            caption: None,
                            location: None,
                            user_date: None,
                            original_date: None,
                            dimensions: None,
                        },
                        Asset {
                            key: "bcd234".to_owned(),
                            checksum: "cafebabe".to_owned(),
                            filename: "img_1234.jpg".to_owned(),
                            byte_length: 1024,
                            media_type: "image/jpeg".to_owned(),
                            tags: vec!["kitten".to_owned(), "puppy".to_owned()],
                            import_date: Utc::now(),
                            caption: None,
                            location: None,
                            user_date: None,
                            original_date: None,
                            dimensions: None,
                        },
                        Asset {
                            key: "cde345".to_owned(),
                            checksum: "cafebabe".to_owned(),
                            filename: "img_1234.jpg".to_owned(),
                            byte_length: 1024,
                            media_type: "image/jpeg".to_owned(),
                            tags: vec!["clouds".to_owned(), "rainbow".to_owned()],
                            import_date: Utc::now(),
                            caption: None,
                            location: None,
                            user_date: None,
                            original_date: None,
                            dimensions: None,
                        },
                    ],
                    cursor: None,
                })
            });
        mock.expect_fetch_assets()
            .withf(|c, _| c.is_some())
            .returning(|_, _| {
                Ok(FetchedAssets {
                    assets: vec![],
                    cursor: None,
                })
            });
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = ScanAssets::new(Box::new(mock), Box::new(cache));
        let params = Params {
            query: "tag:clouds".into(),
            sort_field: None,
            sort_order: None,
        };
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].asset_id, "cde345");
    }

    fn make_fetch_assets() -> Vec<Asset> {
        vec![
            Asset {
                key: "abc123".to_owned(),
                checksum: "cafebabe".to_owned(),
                filename: "img_1234.jpg".to_owned(),
                byte_length: 1024,
                media_type: "image/jpeg".to_owned(),
                tags: vec!["cat".to_owned(), "dog".to_owned()],
                import_date: Utc::now(),
                caption: None,
                location: Some(Location::with_parts("", "Paris", "France")),
                user_date: None,
                original_date: None,
                dimensions: None,
            },
            Asset {
                key: "bcd234".to_owned(),
                checksum: "cafebabe".to_owned(),
                filename: "img_1234.jpg".to_owned(),
                byte_length: 1024,
                media_type: "image/jpeg".to_owned(),
                tags: vec!["kitten".to_owned(), "puppy".to_owned()],
                import_date: Utc::now(),
                caption: None,
                location: Some(Location::with_parts("beach", "Nice", "France")),
                user_date: None,
                original_date: None,
                dimensions: None,
            },
            Asset {
                key: "cde345".to_owned(),
                checksum: "cafebabe".to_owned(),
                filename: "img_1234.jpg".to_owned(),
                byte_length: 1024,
                media_type: "image/jpeg".to_owned(),
                tags: vec!["clouds".to_owned(), "rainbow".to_owned()],
                import_date: Utc::now(),
                caption: None,
                location: Some(Location::with_parts("river", "", "")),
                user_date: None,
                original_date: None,
                dimensions: None,
            },
        ]
    }

    #[test]
    fn test_scan_unset_location_label() {
        // arrange
        let mut mock = MockRecordRepository::new();
        mock.expect_fetch_assets()
            .withf(|c, _| c.is_none())
            .returning(move |_, _| {
                Ok(FetchedAssets {
                    assets: make_fetch_assets(),
                    cursor: None,
                })
            });
        mock.expect_fetch_assets()
            .withf(|c, _| c.is_some())
            .returning(|_, _| {
                Ok(FetchedAssets {
                    assets: vec![],
                    cursor: None,
                })
            });
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = ScanAssets::new(Box::new(mock), Box::new(cache));
        let params = Params {
            query: "loc:label:".into(),
            sort_field: None,
            sort_order: None,
        };
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].asset_id, "abc123");
    }

    #[test]
    fn test_scan_cache_sort_by_date() {
        // arrange
        let mut mock = MockRecordRepository::new();
        mock.expect_fetch_assets()
            .withf(|c, _| c.is_none())
            .times(1)
            .returning(move |_, _| {
                Ok(FetchedAssets {
                    assets: make_fetch_assets(),
                    cursor: Some("cde345".to_owned()),
                })
            });
        mock.expect_fetch_assets()
            .withf(|c, _| c.is_some())
            .times(1)
            .returning(|_, _| {
                Ok(FetchedAssets {
                    assets: vec![],
                    cursor: None,
                })
            });
        let mut cache = MockSearchRepository::new();
        let mut cache_hit = false;
        cache.expect_get().returning(move |_| {
            if cache_hit {
                let assets = make_fetch_assets();
                Ok(Some(vec![SearchResult::new(&assets[1])]))
            } else {
                cache_hit = true;
                Ok(None)
            }
        });
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = ScanAssets::new(Box::new(mock), Box::new(cache));
        let params = Params {
            query: "tag:kitten".into(),
            sort_field: Some(SortField::Date),
            sort_order: Some(SortOrder::Descending),
        };
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].asset_id, "bcd234");

        // act (same search but different sort order, should hit the cache and
        // yet sort the results accordingly)
        let params = Params {
            query: "tag:kitten".into(),
            sort_field: Some(SortField::Date),
            sort_order: Some(SortOrder::Ascending),
        };
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].asset_id, "bcd234");
    }
}

mod query {
    use crate::domain::entities::Asset;
    use anyhow::{anyhow, Error};
    use chrono::{DateTime, NaiveDate, NaiveTime, ParseError, Utc};
    use std::str::FromStr;

    /// Determines if an asset matches certain criteria.
    pub trait Predicate: std::fmt::Debug {
        /// For a given asset, return `true` if the asset matches.
        fn matches(&self, asset: &Asset) -> bool;
    }

    /// Convert a keyword and its arguments into a predicate.
    pub fn build_predicate(atom: Vec<String>) -> Result<Box<dyn Predicate>, Error> {
        let keyword = atom.first().ok_or_else(|| anyhow!("missing keyword"))?;
        if keyword == "after" {
            let arg1 = atom
                .get(1)
                .ok_or_else(|| anyhow!("after requires 1 argument"))?;
            Ok(Box::new(AfterPredicate::new(arg1)?))
        } else if keyword == "before" {
            let arg1 = atom
                .get(1)
                .ok_or_else(|| anyhow!("before requires 1 argument"))?;
            Ok(Box::new(BeforePredicate::new(arg1)?))
        } else if keyword == "is" {
            let arg1 = atom
                .get(1)
                .ok_or_else(|| anyhow!("is requires 1 argument"))?;
            Ok(Box::new(TypePredicate::new(arg1)))
        } else if keyword == "filename" {
            let arg1 = atom
                .get(1)
                .ok_or_else(|| anyhow!("filename requires 1 argument"))?;
            Ok(Box::new(FilenamePredicate::new(arg1)))
        } else if keyword == "format" {
            let arg1 = atom
                .get(1)
                .ok_or_else(|| anyhow!("format requires 1 argument"))?;
            Ok(Box::new(SubtypePredicate::new(arg1)))
        } else if keyword == "loc" {
            if atom.len() == 2 {
                let query = atom.get(1).unwrap();
                Ok(Box::new(LocationPredicate::new(query)))
            } else if atom.len() == 3 {
                let field = LocationField::from_str(atom.get(1).unwrap())?;
                let query = atom.get(2).unwrap();
                Ok(Box::new(LocationPredicate::with_field(field, query)))
            } else {
                Err(anyhow!("loc: requires 1 or 2 arguments"))
            }
        } else if keyword == "tag" {
            let arg1 = atom
                .get(1)
                .ok_or_else(|| anyhow!("tag requires 1 argument"))?;
            Ok(Box::new(TagPredicate::new(arg1)))
        } else {
            Err(anyhow!("unsupported predicate: {}", keyword))
        }
    }

    /// Embodies any type of constraint for filtering assets.
    #[derive(Debug)]
    pub enum Constraint {
        /// Matches if both sides also match.
        And(Box<dyn Predicate>, Box<dyn Predicate>),
        /// Matches if either side matches.
        Or(Box<dyn Predicate>, Box<dyn Predicate>),
        /// Mathces only if child predicate does not match.
        Not(Box<dyn Predicate>),
        /// Matches if the given predicate function returns `true`.
        Lambda(Box<dyn Predicate>),
        /// An empty query that matches nothing.
        Empty,
    }

    impl Predicate for Constraint {
        fn matches(&self, asset: &Asset) -> bool {
            match self {
                Constraint::And(left, right) => left.matches(asset) && right.matches(asset),
                Constraint::Or(left, right) => left.matches(asset) || right.matches(asset),
                Constraint::Not(child) => !child.matches(asset),
                Constraint::Lambda(pred) => pred.matches(asset),
                Constraint::Empty => false,
            }
        }
    }

    /// Matches if the asset file name matches the value.
    #[derive(Debug)]
    pub struct FilenamePredicate(String);

    impl FilenamePredicate {
        pub fn new<S: Into<String>>(name: S) -> Self {
            Self(name.into().to_lowercase())
        }
    }

    impl Predicate for FilenamePredicate {
        fn matches(&self, asset: &Asset) -> bool {
            self.0 == asset.filename.to_lowercase()
        }
    }

    /// Matches if the asset media type matches the value.
    #[derive(Debug)]
    pub struct TypePredicate(String);

    impl TypePredicate {
        pub fn new<S: Into<String>>(type_: S) -> Self {
            Self(type_.into().to_lowercase())
        }
    }

    impl Predicate for TypePredicate {
        fn matches(&self, asset: &Asset) -> bool {
            if let Ok(mime) = asset.media_type.parse::<mime::Mime>() {
                mime.type_().eq(&self.0.as_str())
            } else {
                false
            }
        }
    }

    /// Matches if the asset media subtype matches the value.
    #[derive(Debug)]
    pub struct SubtypePredicate(String);

    impl SubtypePredicate {
        pub fn new<S: Into<String>>(subtype: S) -> Self {
            Self(subtype.into().to_lowercase())
        }
    }

    impl Predicate for SubtypePredicate {
        fn matches(&self, asset: &Asset) -> bool {
            if let Ok(mime) = asset.media_type.parse::<mime::Mime>() {
                mime.subtype().eq(&self.0.as_str())
            } else {
                false
            }
        }
    }

    /// Matches if the asset contains a tag equal to the value.
    #[derive(Debug)]
    pub struct TagPredicate(String);

    impl TagPredicate {
        pub fn new<S: Into<String>>(tag: S) -> Self {
            Self(tag.into().to_lowercase())
        }
    }

    impl Predicate for TagPredicate {
        fn matches(&self, asset: &Asset) -> bool {
            asset.tags.iter().any(|t| t.to_lowercase() == self.0)
        }
    }

    /// Location field on which to match the query.
    ///
    /// Not to be confused with LocationField in the entities module.
    #[derive(Clone, Debug)]
    pub enum LocationField {
        Any,
        Label,
        City,
        Region,
    }

    impl FromStr for LocationField {
        type Err = Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if s == "label" {
                Ok(LocationField::Label)
            } else if s == "city" {
                Ok(LocationField::City)
            } else if s == "region" {
                Ok(LocationField::Region)
            } else if s == "any" {
                Ok(LocationField::Any)
            } else {
                Err(anyhow!("field must be 'any', 'label', 'city', or 'region'"))
            }
        }
    }

    /// Matches if the asset has a location field that matches the value.
    ///
    /// If the value is the empty string, then predicate matches if the
    /// corresponding field is not set.
    #[derive(Debug)]
    pub struct LocationPredicate(LocationField, String);

    impl LocationPredicate {
        pub fn new<S: Into<String>>(location: S) -> Self {
            Self(LocationField::Any, location.into().to_lowercase())
        }

        pub fn with_field<S: Into<String>>(field: LocationField, location: S) -> Self {
            Self(field, location.into().to_lowercase())
        }
    }

    impl Predicate for LocationPredicate {
        fn matches(&self, asset: &Asset) -> bool {
            if self.1.is_empty() {
                // corresponding fields must not have a value
                match self.0 {
                    LocationField::Any => asset
                        .location
                        .as_ref()
                        .map(|l| l.label.is_none() || l.city.is_none() || l.region.is_none())
                        .unwrap_or(false),
                    LocationField::Label => asset
                        .location
                        .as_ref()
                        .map(|l| l.label.is_none())
                        .unwrap_or(false),
                    LocationField::City => asset
                        .location
                        .as_ref()
                        .map(|l| l.city.is_none())
                        .unwrap_or(false),
                    LocationField::Region => asset
                        .location
                        .as_ref()
                        .map(|l| l.region.is_none())
                        .unwrap_or(false),
                }
            } else {
                // corresponding field must have a matching value
                match self.0 {
                    LocationField::Any => asset
                        .location
                        .as_ref()
                        .map(|l| l.partial_match(&self.1))
                        .unwrap_or(false),
                    LocationField::Label => asset
                        .location
                        .as_ref()
                        .map(|l| {
                            l.label
                                .as_ref()
                                .map(|b| b.to_lowercase() == self.1)
                                .unwrap_or(false)
                        })
                        .unwrap_or(false),
                    LocationField::City => asset
                        .location
                        .as_ref()
                        .map(|l| {
                            l.city
                                .as_ref()
                                .map(|c| c.to_lowercase() == self.1)
                                .unwrap_or(false)
                        })
                        .unwrap_or(false),
                    LocationField::Region => asset
                        .location
                        .as_ref()
                        .map(|l| {
                            l.region
                                .as_ref()
                                .map(|r| r.to_lowercase() == self.1)
                                .unwrap_or(false)
                        })
                        .unwrap_or(false),
                }
            }
        }
    }

    /// Matches if the asset "best date" comes _after_ the given date.
    #[derive(Debug)]
    pub struct AfterPredicate(DateTime<Utc>);

    impl AfterPredicate {
        pub fn new(input: &str) -> Result<Self, Error> {
            Ok(Self(parse_datetime(input)?))
        }
    }

    impl Predicate for AfterPredicate {
        fn matches(&self, asset: &Asset) -> bool {
            if let Some(ud) = &asset.user_date {
                ud > &self.0
            } else if let Some(od) = &asset.original_date {
                od > &self.0
            } else {
                asset.import_date > self.0
            }
        }
    }

    /// Matches if the asset "best date" comes _before_ the given date.
    #[derive(Debug)]
    struct BeforePredicate(DateTime<Utc>);

    impl BeforePredicate {
        pub fn new(input: &str) -> Result<Self, Error> {
            Ok(Self(parse_datetime(input)?))
        }
    }

    impl Predicate for BeforePredicate {
        fn matches(&self, asset: &Asset) -> bool {
            if let Some(ud) = &asset.user_date {
                ud < &self.0
            } else if let Some(od) = &asset.original_date {
                od < &self.0
            } else {
                asset.import_date < self.0
            }
        }
    }

    /// A liberal date parser that accepts anything from 2010-08-30T12:30 to just 2010.
    fn parse_datetime(input: &str) -> Result<DateTime<Utc>, Error> {
        if input.contains('T') {
            let parts: Vec<&str> = input.split('T').collect();
            let date = parse_date(parts[0])?;
            let time = parse_time(parts[1])?;
            Ok(date.and_time(time).and_utc())
        } else {
            let ok = parse_date(input).map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())?;
            Ok(ok)
        }
    }

    /// Parse only the date using a liberal parser.
    fn parse_date(input: &str) -> Result<NaiveDate, ParseError> {
        let num_dashes = input.chars().filter(|c| *c == '-').count();
        if num_dashes == 2 {
            NaiveDate::parse_from_str(input, "%Y-%m-%d")
        } else if num_dashes == 1 {
            let padded = format!("{}-01", input);
            NaiveDate::parse_from_str(&padded, "%Y-%m-%d")
        } else {
            let padded = format!("{}-01-01", input);
            NaiveDate::parse_from_str(&padded, "%Y-%m-%d")
        }
    }

    /// Parse only the time using a liberal parser.
    fn parse_time(input: &str) -> Result<NaiveTime, ParseError> {
        let num_colons = input.chars().filter(|c| *c == ':').count();
        if num_colons == 2 {
            NaiveTime::parse_from_str(input, "%H:%M:%S")
        } else if num_colons == 1 {
            let padded = format!("{}:00", input);
            NaiveTime::parse_from_str(&padded, "%H:%M:%S")
        } else {
            let padded = format!("{}:00:00", input);
            NaiveTime::parse_from_str(&padded, "%H:%M:%S")
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::domain::entities::Location;
        use chrono::TimeZone;

        #[test]
        fn test_locationfield_from_str() {
            let result = LocationField::from_str("city");
            assert!(result.is_ok());
            let result = LocationField::from_str("label");
            assert!(result.is_ok());
            let result = LocationField::from_str("region");
            assert!(result.is_ok());
            let result = LocationField::from_str("any");
            assert!(result.is_ok());
            let result = LocationField::from_str("boofar");
            assert!(result.is_err());
        }

        #[test]
        fn test_build_predicate() {
            let result = build_predicate(vec!["loc".into(), "label".into(), "beach".into()]);
            assert!(result.is_ok());
            let result = build_predicate(vec!["loc".into(), "beach".into(), "label".into()]);
            assert!(result.is_err());
            let result = build_predicate(vec!["loc".into(), "beach".into()]);
            assert!(result.is_ok());
            let result = build_predicate(vec!["loc".into(), "label".into(), "".into()]);
            assert!(result.is_ok());
        }

        #[test]
        fn test_query_filename_predicate() {
            let pred_t = FilenamePredicate::new("img_1234.jpg");
            let lambda = Constraint::Lambda(Box::new(pred_t));
            let asset1 = Asset {
                key: "abc123".to_owned(),
                checksum: "cafebabe".to_owned(),
                filename: "IMG_1234.jpg".to_owned(),
                byte_length: 1024,
                media_type: "image/jpeg".to_owned(),
                tags: vec!["cat".to_owned()],
                import_date: Utc::now(),
                caption: None,
                location: None,
                user_date: None,
                original_date: None,
                dimensions: None,
            };
            assert!(lambda.matches(&asset1));

            let pred_t = FilenamePredicate::new("MIV_2326.mov");
            let lambda = Constraint::Lambda(Box::new(pred_t));
            assert!(!lambda.matches(&asset1));
        }

        #[test]
        fn test_query_type_predicate() {
            let pred_t = TypePredicate::new("image");
            let lambda = Constraint::Lambda(Box::new(pred_t));
            let mut asset1 = Asset {
                key: "abc123".to_owned(),
                checksum: "cafebabe".to_owned(),
                filename: "img_1234.jpg".to_owned(),
                byte_length: 1024,
                media_type: "image/jpeg".to_owned(),
                tags: vec!["cat".to_owned()],
                import_date: Utc::now(),
                caption: None,
                location: None,
                user_date: None,
                original_date: None,
                dimensions: None,
            };
            assert!(lambda.matches(&asset1));

            let pred_t = TypePredicate::new("video");
            let lambda = Constraint::Lambda(Box::new(pred_t));
            assert!(!lambda.matches(&asset1));

            asset1.media_type = "foobar".to_owned();
            assert!(!lambda.matches(&asset1));
        }

        #[test]
        fn test_query_subtype_predicate() {
            let pred_t = SubtypePredicate::new("jpeg");
            let lambda = Constraint::Lambda(Box::new(pred_t));
            let mut asset1 = Asset {
                key: "abc123".to_owned(),
                checksum: "cafebabe".to_owned(),
                filename: "img_1234.jpg".to_owned(),
                byte_length: 1024,
                media_type: "image/jpeg".to_owned(),
                tags: vec!["cat".to_owned()],
                import_date: Utc::now(),
                caption: None,
                location: None,
                user_date: None,
                original_date: None,
                dimensions: None,
            };
            assert!(lambda.matches(&asset1));

            let pred_t = SubtypePredicate::new("png");
            let lambda = Constraint::Lambda(Box::new(pred_t));
            assert!(!lambda.matches(&asset1));

            asset1.media_type = "foobar".to_owned();
            assert!(!lambda.matches(&asset1));
        }

        #[test]
        fn test_query_and_constraint() {
            let pred_a = TagPredicate::new("cat");
            let pred_b = LocationPredicate::new("paris");
            let and_c = Constraint::And(Box::new(pred_a), Box::new(pred_b));
            let asset1 = Asset {
                key: "abc123".to_owned(),
                checksum: "cafebabe".to_owned(),
                filename: "img_1234.jpg".to_owned(),
                byte_length: 1024,
                media_type: "image/jpeg".to_owned(),
                tags: vec!["cat".to_owned(), "dog".to_owned()],
                import_date: Utc::now(),
                caption: None,
                location: Some(Location::new("paris")),
                user_date: None,
                original_date: None,
                dimensions: None,
            };
            assert!(and_c.matches(&asset1));

            let pred_a = TagPredicate::new("cat");
            let pred_b = TagPredicate::new("rabbit");
            let and_c = Constraint::And(Box::new(pred_a), Box::new(pred_b));
            assert!(!and_c.matches(&asset1));
        }

        #[test]
        fn test_query_or_constraint() {
            let pred_a = TagPredicate::new("cat");
            let pred_b = LocationPredicate::new("rabbit");
            let or_c = Constraint::Or(Box::new(pred_a), Box::new(pred_b));
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
                user_date: None,
                original_date: None,
                dimensions: None,
            };
            assert!(or_c.matches(&asset1));

            let pred_a = TagPredicate::new("mouse");
            let pred_b = TagPredicate::new("rabbit");
            let or_c = Constraint::Or(Box::new(pred_a), Box::new(pred_b));
            assert!(!or_c.matches(&asset1));
        }

        #[test]
        fn test_query_not_constraint() {
            let pred_a = LocationPredicate::new("london");
            let not_c = Constraint::Not(Box::new(pred_a));
            let asset1 = Asset {
                key: "abc123".to_owned(),
                checksum: "cafebabe".to_owned(),
                filename: "img_1234.jpg".to_owned(),
                byte_length: 1024,
                media_type: "image/jpeg".to_owned(),
                tags: vec!["cat".to_owned(), "dog".to_owned()],
                import_date: Utc::now(),
                caption: None,
                location: Some(Location::new("paris")),
                user_date: None,
                original_date: None,
                dimensions: None,
            };
            assert!(not_c.matches(&asset1));

            let pred_a = TagPredicate::new("cat");
            let not_c = Constraint::Not(Box::new(pred_a));
            assert!(!not_c.matches(&asset1));
        }

        #[test]
        fn test_query_tag_predicate() {
            let pred = TagPredicate::new("cat");
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
                user_date: None,
                original_date: None,
                dimensions: None,
            };
            assert!(pred.matches(&asset1));
            let pred = TagPredicate::new("dog");
            assert!(pred.matches(&asset1));
            let pred = TagPredicate::new("DOG");
            assert!(pred.matches(&asset1));
            let pred = TagPredicate::new("rabbit");
            assert!(!pred.matches(&asset1));
        }

        #[test]
        fn test_query_location_predicate() {
            let pred = LocationPredicate::new("paris");
            let asset1 = Asset {
                key: "abc123".to_owned(),
                checksum: "cafebabe".to_owned(),
                filename: "img_1234.jpg".to_owned(),
                byte_length: 1024,
                media_type: "image/jpeg".to_owned(),
                tags: vec!["tower".to_owned()],
                import_date: Utc::now(),
                caption: None,
                location: Some(Location::with_parts("Eiffel Tower", "Paris", "France")),
                user_date: None,
                original_date: None,
                dimensions: None,
            };
            assert!(pred.matches(&asset1));
            let pred = LocationPredicate::new("france");
            assert!(pred.matches(&asset1));
            let pred = LocationPredicate::with_field(LocationField::Region, "france");
            assert!(pred.matches(&asset1));
            let pred = LocationPredicate::with_field(LocationField::Region, "paris");
            assert!(!pred.matches(&asset1));
            let pred = LocationPredicate::with_field(LocationField::City, "paris");
            assert!(pred.matches(&asset1));
            let pred = LocationPredicate::new("eiffel tower");
            assert!(pred.matches(&asset1));
            let pred = LocationPredicate::with_field(LocationField::Label, "eiffel tower");
            assert!(pred.matches(&asset1));
            let pred = LocationPredicate::new("texas");
            assert!(!pred.matches(&asset1));
        }

        #[test]
        fn test_query_location_predicate_empty() {
            let pred = LocationPredicate::new("paris");
            let asset1 = Asset {
                key: "abc123".to_owned(),
                checksum: "cafebabe".to_owned(),
                filename: "img_1234.jpg".to_owned(),
                byte_length: 1024,
                media_type: "image/jpeg".to_owned(),
                tags: vec!["tower".to_owned()],
                import_date: Utc::now(),
                caption: None,
                location: Some(Location::with_parts("", "Paris", "France")),
                user_date: None,
                original_date: None,
                dimensions: None,
            };
            assert!(pred.matches(&asset1));
            let pred = LocationPredicate::new("france");
            assert!(pred.matches(&asset1));
            let pred = LocationPredicate::with_field(LocationField::Region, "france");
            assert!(pred.matches(&asset1));
            let pred = LocationPredicate::with_field(LocationField::Region, "paris");
            assert!(!pred.matches(&asset1));
            let pred = LocationPredicate::with_field(LocationField::City, "paris");
            assert!(pred.matches(&asset1));
            let pred = LocationPredicate::new("");
            assert!(pred.matches(&asset1));
            let pred = LocationPredicate::with_field(LocationField::Label, "");
            assert!(pred.matches(&asset1));
            let pred = LocationPredicate::new("texas");
            assert!(!pred.matches(&asset1));
        }

        #[test]
        fn test_query_parse_datetime() {
            let actual = parse_datetime("2010-08-30T12:15:30").unwrap();
            let expected = Utc
                .with_ymd_and_hms(2010, 8, 30, 12, 15, 30)
                .single()
                .unwrap();
            assert_eq!(actual, expected);

            let actual = parse_datetime("2010-08-30T12:15").unwrap();
            let expected = Utc
                .with_ymd_and_hms(2010, 8, 30, 12, 15, 0)
                .single()
                .unwrap();
            assert_eq!(actual, expected);

            let actual = parse_datetime("2010-08-30T12").unwrap();
            let expected = Utc
                .with_ymd_and_hms(2010, 8, 30, 12, 0, 0)
                .single()
                .unwrap();
            assert_eq!(actual, expected);

            let actual = parse_datetime("2010-08-30").unwrap();
            let expected = Utc.with_ymd_and_hms(2010, 8, 30, 0, 0, 0).single().unwrap();
            assert_eq!(actual, expected);

            let actual = parse_datetime("2010-08").unwrap();
            let expected = Utc.with_ymd_and_hms(2010, 8, 1, 0, 0, 0).single().unwrap();
            assert_eq!(actual, expected);

            let actual = parse_datetime("2010").unwrap();
            let expected = Utc.with_ymd_and_hms(2010, 1, 1, 0, 0, 0).single().unwrap();
            assert_eq!(actual, expected);
        }

        #[test]
        fn test_query_after_predicate() {
            let pred = AfterPredicate::new("2010-08-30").unwrap();
            let earlier = Utc.with_ymd_and_hms(2009, 8, 30, 0, 0, 0).single().unwrap();
            let later = Utc.with_ymd_and_hms(2010, 9, 1, 0, 0, 0).single().unwrap();
            let mut asset1 = Asset {
                key: "abc123".to_owned(),
                checksum: "cafebabe".to_owned(),
                filename: "img_1234.jpg".to_owned(),
                byte_length: 1024,
                media_type: "image/jpeg".to_owned(),
                tags: vec!["cat".to_owned(), "dog".to_owned()],
                import_date: earlier,
                caption: None,
                location: None,
                user_date: None,
                original_date: None,
                dimensions: None,
            };
            assert!(!pred.matches(&asset1));
            asset1.import_date = later;
            assert!(pred.matches(&asset1));

            asset1.import_date = earlier;
            asset1.original_date = Some(later);
            assert!(pred.matches(&asset1));

            asset1.original_date = None;
            asset1.user_date = Some(later);
            assert!(pred.matches(&asset1));
        }

        #[test]
        fn test_query_before_predicate() {
            let pred = BeforePredicate::new("2010-08-30").unwrap();
            let earlier = Utc.with_ymd_and_hms(2009, 8, 30, 0, 0, 0).single().unwrap();
            let later = Utc.with_ymd_and_hms(2010, 9, 1, 0, 0, 0).single().unwrap();
            let mut asset1 = Asset {
                key: "abc123".to_owned(),
                checksum: "cafebabe".to_owned(),
                filename: "img_1234.jpg".to_owned(),
                byte_length: 1024,
                media_type: "image/jpeg".to_owned(),
                tags: vec!["cat".to_owned(), "dog".to_owned()],
                import_date: later,
                caption: None,
                location: None,
                user_date: None,
                original_date: None,
                dimensions: None,
            };
            assert!(!pred.matches(&asset1));
            asset1.import_date = earlier;
            assert!(pred.matches(&asset1));

            asset1.import_date = later;
            asset1.original_date = Some(earlier);
            assert!(pred.matches(&asset1));

            asset1.original_date = None;
            asset1.user_date = Some(earlier);
            assert!(pred.matches(&asset1));
        }
    }
}

mod parser {
    //!
    //! A simple parser for the query language, modeled after that of the
    //! perkeep application (https://perkeep.org).
    //!

    use super::lexer::{Token, TokenType};
    use super::query::{build_predicate, Constraint};
    use crate::domain::usecases::scan::lexer;
    use anyhow::{anyhow, Error};
    use std::sync::mpsc::Receiver;

    /// Parse the given query and return a constraint for filtering assets.
    pub fn parse_query(query: &str) -> Result<Constraint, Error> {
        let mut parser = Parser::new(query);
        let result = parser.parse_exp();
        if let Ok(last) = parser.next() {
            parser.drain_lexer();
            if result.is_ok() && last.typ != TokenType::Eof {
                return Err(anyhow!("trailing tokens: {}", last));
            }
        }
        result
    }

    pub struct Parser {
        tokens: Receiver<Token>,
        peeked: Option<Token>,
    }

    impl Parser {
        fn new(expr: &str) -> Self {
            let rx = lexer::lex(expr);
            Self {
                tokens: rx,
                peeked: None,
            }
        }

        // Ensure all tokens are read from the channel such that the lexer
        // thread can exit properly.
        fn drain_lexer(&mut self) {
            let _: Vec<Token> = self.tokens.iter().collect();
        }

        fn next(&mut self) -> Result<Token, Error> {
            if let Some(token) = self.peeked.take() {
                Ok(token)
            } else {
                Ok(self.tokens.recv()?)
            }
        }

        fn peek(&mut self) -> Result<Token, Error> {
            if self.peeked.is_none() {
                self.peeked = Some(self.tokens.recv()?);
            }
            Ok(self.peeked.as_ref().unwrap().clone())
        }

        /// Parse the query from beginning to end, which includes expressions
        /// wrapped in parentheses.
        fn parse_exp(&mut self) -> Result<Constraint, Error> {
            if let Ok(p) = self.peek() {
                if p.typ == TokenType::Eof {
                    return Ok(Constraint::Empty);
                }
            }
            let mut ret = self.parse_operand()?;
            loop {
                let p = self.peek()?;
                if p.typ == TokenType::And {
                    self.next()?;
                } else if p.typ == TokenType::Or {
                    self.next()?;
                    return self.parse_or_rhs(ret);
                } else if p.typ == TokenType::Close || p.typ == TokenType::Eof {
                    break;
                }
                ret = self.parse_and_rhs(ret)?;
            }
            Ok(ret)
        }

        /// Process the next token as an operand, returning an error if it is
        /// anything else.
        fn parse_operand(&mut self) -> Result<Constraint, Error> {
            // negated := p.stripNot()
            let negated = self.strip_not()?;
            let mut ret = Constraint::Empty;
            let op = self.peek()?;
            if op.typ == TokenType::Error {
                return Err(anyhow!("error: {}", op.val));
            } else if op.typ == TokenType::Eof {
                return Err(anyhow!("error: expected operand, got {}", op.val));
            } else if op.typ == TokenType::Close {
                return Err(anyhow!("error: found ) without (, got {}", op.val));
            } else if op.typ == TokenType::Predicate
                || op.typ == TokenType::Colon
                || op.typ == TokenType::Arg
            {
                ret = self.parse_atom()?;
            } else if op.typ == TokenType::Open {
                ret = self.parse_group()?;
            }
            if negated {
                ret = Constraint::Not(Box::new(ret));
            }
            Ok(ret)
        }

        /// Processes consecuivee `not` operators, returning `true` if an odd
        /// number and `false` otherwise.
        fn strip_not(&mut self) -> Result<bool, Error> {
            let mut negated = false;
            loop {
                let p = self.peek()?;
                if p.typ == TokenType::Not {
                    self.next()?;
                    negated = !negated;
                    continue;
                }
                break;
            }
            Ok(negated)
        }

        /// Current token is expected to be a predicate, followed by a colon,
        /// and one or more arguments separated by colons. The predicate will be
        /// converted to one of the supported predicates.
        fn parse_atom(&mut self) -> Result<Constraint, Error> {
            let mut i = self.peek()?;
            let mut a: Vec<String> = vec![];
            // confirm that the first token is a predicate, everything else is wrong
            if i.typ == TokenType::Predicate {
                self.next()?;
                a.push(i.val);
            } else {
                return Err(anyhow!("expected predicate, got {}", i));
            }
            let mut arg_expected = false;
            loop {
                i = self.peek()?;
                if i.typ == TokenType::Colon {
                    arg_expected = true;
                    self.next()?;
                    continue;
                } else if i.typ == TokenType::Arg {
                    arg_expected = false;
                    i = self.next()?;
                    a.push(i.val);
                    continue;
                }
                if arg_expected {
                    // inject an empty argument after the trailing colon
                    a.push("".into());
                }
                break;
            }
            build_predicate(a).map(Constraint::Lambda)
        }

        /// Current token is expected to be an open paren.
        fn parse_group(&mut self) -> Result<Constraint, Error> {
            // confirm the next token is an open paren
            let i = self.next()?;
            if i.typ == TokenType::Open {
                let c = self.parse_exp()?;
                let p = self.peek()?;
                if p.typ == TokenType::Close {
                    self.next()?;
                    return Ok(c);
                }
                return Err(anyhow!("no matching ) at {}", i));
            }
            Err(anyhow!("expected ( but got {}", i))
        }

        /// Process the right side of the `or`, including chained `or` operators.
        fn parse_or_rhs(&mut self, lhs: Constraint) -> Result<Constraint, Error> {
            let mut ret = lhs;
            loop {
                let rhs = self.parse_and()?;
                ret = Constraint::Or(Box::new(ret), Box::new(rhs));
                let p = self.peek()?;
                if p.typ == TokenType::Or {
                    self.next()?;
                } else if p.typ == TokenType::And
                    || p.typ == TokenType::Close
                    || p.typ == TokenType::Eof
                {
                    break;
                }
            }
            Ok(ret)
        }

        /// Process the `and` and whatever comes after it.
        fn parse_and(&mut self) -> Result<Constraint, Error> {
            let ret = self.parse_operand()?;
            let p = self.peek()?;
            if p.typ == TokenType::And {
                self.next()?;
            } else if p.typ == TokenType::Or || p.typ == TokenType::Close || p.typ == TokenType::Eof
            {
                return Ok(ret);
            }
            self.parse_and_rhs(ret)
        }

        /// Process the right side of the `and`, including chained `and` operators.
        fn parse_and_rhs(&mut self, lhs: Constraint) -> Result<Constraint, Error> {
            let mut ret = lhs;
            loop {
                let rhs = self.parse_operand()?;
                ret = Constraint::And(Box::new(ret), Box::new(rhs));
                let p = self.peek()?;
                if p.typ == TokenType::And {
                    self.next()?;
                    continue;
                }
                break;
            }
            Ok(ret)
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_parser_empty_query() {
            let result = parse_query("");
            assert!(result.is_ok());
            let cons = result.unwrap();
            assert!(matches!(cons, Constraint::Empty));
        }

        #[test]
        fn test_parser_one_predicate() {
            let result = parse_query("tag:kittens");
            assert!(result.is_ok());
            let cons = result.unwrap();
            assert!(matches!(cons, Constraint::Lambda(_)));
        }

        #[test]
        fn test_parser_complex_predicate() {
            let result = parse_query("loc:city:paris");
            assert!(result.is_ok());
            let cons = result.unwrap();
            assert!(matches!(cons, Constraint::Lambda(_)));

            let result = parse_query("loc:city:");
            assert!(result.is_ok());
            let cons = result.unwrap();
            assert!(matches!(cons, Constraint::Lambda(_)));
        }

        #[test]
        fn test_parser_not_one_predicate() {
            // whitespace around `-` (not) is ignored
            let result = parse_query(" - tag:kittens");
            assert!(result.is_ok());
            let ac = result.unwrap();
            assert!(matches!(ac, Constraint::Not(_)));
        }

        #[test]
        fn test_parser_double_negatives() {
            // even number of not operators cancel out
            let result = parse_query("--tag:kittens");
            assert!(result.is_ok());
            let ac = result.unwrap();
            assert!(matches!(ac, Constraint::Lambda(_)));
        }

        #[test]
        fn test_parser_and_two_predicates() {
            let result = parse_query("after:2003-08-30 and before:2004-08-31");
            assert!(result.is_ok());
            let ac = result.unwrap();
            assert!(matches!(ac, Constraint::And(_, _)));
        }

        #[test]
        fn test_parser_or_two_predicates() {
            let result = parse_query("tag:food or loc:paris");
            assert!(result.is_ok());
            let ac = result.unwrap();
            assert!(matches!(ac, Constraint::Or(_, _)));
        }

        #[test]
        fn test_parser_groups_and_or() {
            let result = parse_query("(tag:food or tag:clothes) and loc:paris");
            assert!(result.is_ok());
            let ac = result.unwrap();
            // the `and` takes precedence due to grouping
            assert!(matches!(ac, Constraint::And(_, _)));
        }

        #[test]
        fn test_parser_unsupported_keyword_and_more() {
            // if more tokens follow a parsing error, the error should be
            // returned rather than failing because there are trailing tokens
            let result = parse_query("orc:bit or loc:paris");
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.to_string(), "unsupported predicate: orc");
        }
    }
}

mod lexer {
    //!
    //! A lexical analyzer for the simple query language.
    //!
    //! Fashioned after that which was presented by Rob Pike in the "Lexical
    //! Scanning in Go" talk (https://go.dev/talks/2011/lex.slide). The general
    //! idea is that the lexer produces tokens and sends them to a channel, from
    //! which a parser would consume them.
    //!
    //! The design of the lexer involves a finite state machine consisting of
    //! function pointers. The starting function determines which function
    //! should go next, returning the pointer to that function. This continues
    //! until either `None` is returned by a function, or the end of the input
    //! is reached. The "machine" itself is very simple, it continuously invokes
    //! the current state function, using its return value as the next function
    //! to invoke.
    //!
    //! As each function processes the input, it may emit one or more tokens.
    //! These are sent over a channel from which the recipient, presumably a
    //! parser, consumes them. The lexer runs in a separate thread, sending
    //! tokens to the channel until either it fills up and blocks, or the input
    //! is exhausted.
    //!

    use std::char;
    use std::fmt;
    use std::str::CharIndices;
    use std::sync::mpsc::{self, Receiver, SyncSender};
    use std::thread;

    const WHITESPACE: &str = "\t\n\r ";
    // operator boundary
    const OP_BOUND: &str = "\t\n\r (";

    /// Defines the type of a particular token.
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub enum TokenType {
        And,
        Arg,
        Close,
        Colon,
        Eof,
        Error,
        Not,
        Open,
        Or,
        Predicate,
    }

    impl fmt::Display for TokenType {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                TokenType::And => write!(f, "And"),
                TokenType::Arg => write!(f, "Arg"),
                TokenType::Close => write!(f, "Close"),
                TokenType::Colon => write!(f, "Colon"),
                TokenType::Eof => write!(f, "EOF"),
                TokenType::Error => write!(f, "Error"),
                TokenType::Not => write!(f, "Not"),
                TokenType::Open => write!(f, "Open"),
                TokenType::Or => write!(f, "Or"),
                TokenType::Predicate => write!(f, "Predicate"),
            }
        }
    }

    /// Represents a single token emitted by the lexer.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Token {
        pub typ: TokenType,
        pub val: String,
    }

    impl fmt::Display for Token {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Token[{}: '{}']", self.typ, self.val)
        }
    }

    /// The `Lexer` struct holds the state of the lexical analyzer.
    struct Lexer<'a> {
        // the string being scanned
        input: &'a str,
        // iterator of the characters in the string
        iter: CharIndices<'a>,
        // the next character to return, if peek() has been called
        peeked: Option<(usize, char)>,
        // start position of the current token (in bytes)
        start: usize,
        // position of next character to read (in bytes)
        pos: usize,
        // width of last character read from input (in bytes)
        width: usize,
        // channel sender for scanned tokens
        chan: SyncSender<Token>,
    }

    impl<'a> fmt::Display for Lexer<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Lexer at offset {}", self.pos)
        }
    }

    impl<'a> Lexer<'a> {
        /// `new` constructs an instance of `Lexer` for the named input.
        fn new(input: &'a str, chan: SyncSender<Token>) -> Lexer<'a> {
            Lexer {
                input,
                iter: input.char_indices(),
                peeked: None,
                start: 0,
                pos: 0,
                width: 0,
                chan,
            }
        }

        /// emit passes the current token back to the client via the channel.
        fn emit(&mut self, t: TokenType) {
            let text = &self.input[self.start..self.pos];
            let _ = self.chan.send(Token {
                typ: t,
                val: text.to_string(),
            });
            self.start = self.pos;
        }

        /// `emit_error` passes the message back to the client via the channel.
        fn emit_error(&mut self, msg: &str) {
            let _ = self.chan.send(Token {
                typ: TokenType::Error,
                val: msg.to_string(),
            });
            self.start = self.pos;
        }

        /// `emit_string` passes the given token back to the client via the channel.
        fn emit_string(&mut self, t: TokenType, text: &str) {
            let _ = self.chan.send(Token {
                typ: t,
                val: text.to_string(),
            });
            self.start = self.pos;
        }

        /// `next` returns the next rune in the input, or `None` if at the end.
        fn next(&mut self) -> Option<char> {
            let next = if self.peeked.is_some() {
                self.peeked.take()
            } else {
                self.iter.next()
            };
            match next {
                Some((pos, ch)) => {
                    self.width = ch.len_utf8();
                    self.pos = pos + self.width;
                    Some(ch)
                }
                None => None,
            }
        }

        /// `peek` returns but does not consume the next rune in the input.
        fn peek(&mut self) -> Option<char> {
            if self.peeked.is_none() {
                self.peeked = self.iter.next();
            }
            self.peeked.map(|(_, ch)| ch)
        }

        /// `ignore` skips over the pending input before this point.
        fn ignore(&mut self) {
            self.start = self.pos;
        }

        /// `rewind` moves the current position back to the start of the current token.
        fn rewind(&mut self) {
            self.pos = self.start;
            self.width = 0;
            self.peeked = None;
            self.iter = self.input.char_indices();
            for _ in 0..self.start {
                self.iter.next();
            }
        }

        /// `is_match` returns `true` if the next rune is from the valid set.
        /// The character is not consumed either way.
        fn is_match(&mut self, valid: &str) -> bool {
            if let Some(ch) = self.peek() {
                valid.contains(ch)
            } else {
                false
            }
        }

        /// `accept_string` consumes the next set of characters if they match
        /// the input string, otherwise rewinds and returns `false`.
        fn accept_string(&mut self, s: &str) -> bool {
            for r in s.chars() {
                if self.next() != Some(r) {
                    self.rewind();
                    return false;
                }
            }
            true
        }

        /// `accept_run` consumes a run of runes from the valid set.
        fn accept_run(&mut self, valid: &str) -> bool {
            let old_pos = self.pos;
            while let Some(ch) = self.peek() {
                if valid.contains(ch) {
                    // consume the character
                    let _ = self.next();
                } else {
                    break;
                }
            }
            old_pos < self.pos
        }

        /// `accept_run_fn` consumes a run of runes until the function returns `false`.
        fn accept_run_fn(&mut self, valid: fn(char) -> bool) -> bool {
            let old_pos = self.pos;
            while let Some(ch) = self.peek() {
                if valid(ch) {
                    // consume the character
                    let _ = self.next();
                } else {
                    break;
                }
            }
            old_pos < self.pos
        }
    }

    /// `StateFn` represents the state of the scanner as a function that returns
    /// the next state. As a side effect of the function, tokens may be emitted.
    /// Cannot use recursive types, as in Go, so must wrap in a struct.
    struct StateFn(fn(&mut Lexer) -> Option<StateFn>);

    /// Initiates the lexical analysis of the given input text.
    ///
    /// The lex function initializes the lexer to analyze the given query,
    /// returning the channel receiver from which tokens are received.
    pub fn lex(input: &str) -> Receiver<Token> {
        let owned = input.to_owned();
        let (tx, rx) = mpsc::sync_channel(1);

        thread::spawn(move || {
            let mut lexer = Lexer::new(&owned, tx);
            // inform the compiler what the type of state _really_ is
            let mut state: fn(&mut Lexer) -> Option<StateFn> = lex_start;
            while let Some(next) = state(&mut lexer) {
                let StateFn(state_fn) = next;
                state = state_fn;
            }
        });
        rx
    }

    /// `errorf` emits an error token and returns `None` to end lexing.
    fn errorf(l: &mut Lexer, message: &str) -> Option<StateFn> {
        l.emit_error(message);
        None
    }

    /// `lex_start` reads the next token from the input and determines what
    /// to do with that token, returning the appropriate state function.
    fn lex_start(l: &mut Lexer) -> Option<StateFn> {
        l.accept_run(WHITESPACE);
        l.ignore();
        if let Some(ch) = l.next() {
            match ch {
                '(' => {
                    l.emit(TokenType::Open);
                    Some(StateFn(lex_start))
                }
                ')' => {
                    l.emit(TokenType::Close);
                    Some(StateFn(lex_operator))
                }
                '-' => {
                    l.emit(TokenType::Not);
                    Some(StateFn(lex_start))
                }
                _ => {
                    l.rewind();
                    Some(StateFn(lex_predicate))
                }
            }
        } else {
            l.emit(TokenType::Eof);
            None
        }
    }

    /// `lex_operator` expects to find a boolean operator such as "and" or "or".
    fn lex_operator(l: &mut Lexer) -> Option<StateFn> {
        l.accept_run(WHITESPACE);
        l.ignore();
        match l.peek() {
            Some('a') => Some(StateFn(lex_and)),
            Some('o') => Some(StateFn(lex_or)),
            _ => Some(StateFn(lex_start)),
        }
    }

    /// `lex_and` expects to find 'and' followed by whitespace or open paren.
    fn lex_and(l: &mut Lexer) -> Option<StateFn> {
        if l.accept_string("and") && l.is_match(OP_BOUND) {
            l.emit(TokenType::And);
            return Some(StateFn(lex_start));
        }
        Some(StateFn(lex_predicate))
    }

    /// `lex_or` expects to find 'or' followed by whitespace or open paren.
    fn lex_or(l: &mut Lexer) -> Option<StateFn> {
        if l.accept_string("or") && l.is_match(OP_BOUND) {
            l.emit(TokenType::Or);
            return Some(StateFn(lex_start));
        }
        Some(StateFn(lex_predicate))
    }

    /// `lex_predicate` expects to read an alphabetic string followed by a colon
    /// (:), otherwise an error is emitted.
    fn lex_predicate(l: &mut Lexer) -> Option<StateFn> {
        l.accept_run_fn(char::is_alphabetic);
        if let Some(ch) = l.peek() {
            if ch == ':' {
                l.emit(TokenType::Predicate);
                l.next();
                l.emit(TokenType::Colon);
                return Some(StateFn(lex_argument));
            }
        }
        errorf(l, "bare literals unsupported")
    }

    /// `lex_argument` processes double-quoted strings, single-quoted strings,
    /// and raw values, including chains of arguments separated by colons.
    fn lex_argument(l: &mut Lexer) -> Option<StateFn> {
        if let Some(ch) = l.next() {
            if ch == '"' {
                return Some(StateFn(lex_string_double));
            } else if ch == '\'' {
                return Some(StateFn(lex_string_single));
            }
            // anything else must be a raw value
            l.rewind();
            l.accept_run_fn(is_search_word_rune);
            l.emit(TokenType::Arg);
            if let Some(ch) = l.peek() {
                if ch == ':' {
                    l.next();
                    l.emit(TokenType::Colon);
                    return Some(StateFn(lex_argument));
                }
            }
            return Some(StateFn(lex_operator));
        }
        // ran out of tokens
        Some(StateFn(lex_start))
    }

    /// `lex_string_double` expects the current character to be a double-quote
    /// and scans the input to find the end of the quoted string.
    fn lex_string_double(l: &mut Lexer) -> Option<StateFn> {
        let mut text = String::new();
        while let Some(ch) = l.next() {
            match ch {
                // pass over escaped characters
                '\\' => {
                    if let Some(ch) = l.next() {
                        match ch {
                            '"' | '\'' | ' ' | '\t' => text.push(ch),
                            _ => {
                                // otherwise let replace_escapes() handle it
                                text.push('\\');
                                text.push(ch);
                            }
                        }
                    } else {
                        return errorf(l, "improperly terminated string");
                    }
                }
                '"' => {
                    // reached the end of the string
                    match replace_escapes(&text[..]) {
                        Ok(escaped) => {
                            l.emit_string(TokenType::Arg, &escaped[..]);
                            return Some(StateFn(lex_operator));
                        }
                        Err(msg) => {
                            return errorf(l, msg);
                        }
                    }
                }
                _ => {
                    text.push(ch);
                }
            }
        }
        errorf(l, "unclosed quoted string")
    }

    /// `lex_string_single` expects the current character to be a single-quote
    /// and scans the input to find the end of the quoted string.
    fn lex_string_single(l: &mut Lexer) -> Option<StateFn> {
        let mut text = String::new();
        while let Some(ch) = l.next() {
            match ch {
                // pass over escaped characters
                '\\' => {
                    if let Some(ch) = l.next() {
                        match ch {
                            '"' | '\'' | ' ' | '\t' => text.push(ch),
                            _ => {
                                // otherwise let replace_escapes() handle it
                                text.push('\\');
                                text.push(ch);
                            }
                        }
                    } else {
                        return errorf(l, "improperly terminated string");
                    }
                }
                '\'' => {
                    // reached the end of the string
                    match replace_escapes(&text[..]) {
                        Ok(escaped) => {
                            l.emit_string(TokenType::Arg, &escaped[..]);
                            return Some(StateFn(lex_operator));
                        }
                        Err(msg) => {
                            return errorf(l, msg);
                        }
                    }
                }
                _ => {
                    text.push(ch);
                }
            }
        }
        errorf(l, "unclosed quoted string")
    }

    /// `is_search_word_rune` defines those characters that are part of an
    /// unquoted argument, which includes non-whitespace and the symbols
    /// supported by the lexer (colon, parentheses).
    fn is_search_word_rune(ch: char) -> bool {
        if ch == ':' || ch == '(' || ch == ')' {
            return false;
        }
        !ch.is_whitespace()
    }

    /// `replace_escapes` replaces any \xNNNN; escape sequences with the Unicode
    /// code point identified by the NNNN hexadecimal value, where NNNN can be
    /// two, three, or four hexadecimal digits. The code point must be valid.
    /// Also handles the \a, \b, \t, \n, and \r escapes.
    fn replace_escapes(text: &str) -> Result<String, &'static str> {
        let mut result = String::new();
        let mut iter = text.chars();
        while let Some(ch) = iter.next() {
            if ch == '\\' {
                if let Some(ch) = iter.next() {
                    match ch {
                        'a' => result.push('\x07'),
                        'b' => result.push('\x08'),
                        't' => result.push('\t'),
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        '\\' => result.push('\\'),
                        'x' => {
                            let mut hex = String::new();
                            loop {
                                if let Some(ch) = iter.next() {
                                    if ch == ';' {
                                        break;
                                    }
                                    hex.push(ch);
                                } else {
                                    return Err("missing ; after \\x escape sequence");
                                }
                            }
                            // verify this is a valid inline hex escape value
                            match u32::from_str_radix(&hex[..], 16) {
                                Ok(code) => match char::from_u32(code) {
                                    Some(x) => result.push(x),
                                    None => {
                                        return Err("invalid UTF code point");
                                    }
                                },
                                Err(_) => {
                                    return Err("invalid hexadecimal escape code");
                                }
                            }
                        }
                        _ => {
                            return Err("expected x|a|b|t|n|r after \\ in escape sequence");
                        }
                    }
                } else {
                    return Err("reached EOF after \\ escape");
                }
            } else {
                result.push(ch);
            }
        }
        Ok(result)
    }

    #[cfg(test)]
    mod test {
        use super::{lex, replace_escapes, TokenType};
        use std::vec::Vec;

        /// `verify_success` lexes a query and verifies that the tokens
        /// emitted match those in the vector of tuples.
        fn verify_success(input: &str, expected: Vec<(TokenType, &str)>) {
            let rx = lex(input);
            for er in expected.iter() {
                if let Ok(token) = rx.recv() {
                    assert_eq!(token.typ, er.0, "{}", token);
                    assert_eq!(token.val, er.1, "{}", token);
                    println!("token ok -> {}", token);
                } else {
                    panic!("ran out of tokens");
                }
            }
            // make sure we have reached the end of the results
            if let Ok(token) = rx.recv() {
                assert_eq!(token.typ, TokenType::Eof);
            } else {
                panic!("should have exhausted tokens");
            }
        }

        #[test]
        fn test_lexer_replace_escapes() {
            // normal cases
            assert_eq!(
                replace_escapes("foo bar baz quux").unwrap(),
                "foo bar baz quux".to_string()
            );
            assert_eq!(
                replace_escapes("foo\\x20;quux").unwrap(),
                "foo quux".to_string()
            );
            assert_eq!(
                replace_escapes("\\x65e5;\\x672c;\\x8a9e;").unwrap(),
                "日本語".to_string()
            );
            assert_eq!(replace_escapes("\\a").unwrap(), "\x07".to_string());
            assert_eq!(replace_escapes("\\b").unwrap(), "\x08".to_string());
            assert_eq!(replace_escapes("\\t").unwrap(), "\t".to_string());
            assert_eq!(replace_escapes("\\n").unwrap(), "\n".to_string());
            assert_eq!(replace_escapes("\\r").unwrap(), "\r".to_string());
            // error cases
            assert_eq!(
                replace_escapes("\\f").unwrap_err(),
                "expected x|a|b|t|n|r after \\ in escape sequence"
            );
            assert_eq!(
                replace_escapes("\\xAB").unwrap_err(),
                "missing ; after \\x escape sequence"
            );
            assert_eq!(
                replace_escapes("\\xD801;").unwrap_err(),
                "invalid UTF code point"
            );
            assert_eq!(
                replace_escapes("\\xGGGG;").unwrap_err(),
                "invalid hexadecimal escape code"
            );
            assert_eq!(
                replace_escapes("\\").unwrap_err(),
                "reached EOF after \\ escape"
            );
        }

        #[test]
        fn test_lexer_empty_input() {
            let rx = lex("");
            if let Ok(token) = rx.recv() {
                assert_eq!(token.typ, TokenType::Eof);
            } else {
                panic!();
            }
            let rx = lex("   \r  \n   \t  ");
            if let Ok(token) = rx.recv() {
                assert_eq!(token.typ, TokenType::Eof);
            } else {
                panic!();
            }
        }

        #[test]
        fn test_lexer_separators_ignored() {
            let vec = vec![(TokenType::Open, "("), (TokenType::Close, ")")];
            verify_success("     (\n\t )\r\n", vec);
        }

        #[test]
        fn test_lexer_open_close_paren() {
            let vec = vec![(TokenType::Open, "("), (TokenType::Close, ")")];
            verify_success("()", vec);
        }

        #[test]
        fn test_lexer_basic_predicates() {
            let vec = vec![
                (TokenType::Predicate, "tag"),
                (TokenType::Colon, ":"),
                (TokenType::Arg, "kittens"),
                (TokenType::Not, "-"),
                (TokenType::Predicate, "tag"),
                (TokenType::Colon, ":"),
                (TokenType::Arg, "clouds"),
                (TokenType::Predicate, "loc"),
                (TokenType::Colon, ":"),
                (TokenType::Arg, "castro valley"),
                (TokenType::Predicate, "loc"),
                (TokenType::Colon, ":"),
                (TokenType::Arg, "lower manhatten"),
            ];
            verify_success(
                "tag:kittens -tag:clouds loc:'castro valley' loc:\"lower manhatten\"",
                vec,
            );
        }

        #[test]
        fn test_lexer_complex_predicates() {
            let vec = vec![
                (TokenType::Predicate, "loc"),
                (TokenType::Colon, ":"),
                (TokenType::Arg, "city"),
                (TokenType::Colon, ":"),
                (TokenType::Arg, "london"),
                (TokenType::Or, "or"),
                (TokenType::Predicate, "loc"),
                (TokenType::Colon, ":"),
                (TokenType::Arg, "region"),
                (TokenType::Colon, ":"),
                (TokenType::Arg, "japan"),
                (TokenType::Predicate, "loc"),
                (TokenType::Colon, ":"),
                (TokenType::Arg, "label"),
                (TokenType::Colon, ":"),
            ];
            verify_success("loc:city:london or loc:region:japan loc:label:", vec);
        }

        #[test]
        fn test_lexer_basic_operators() {
            let vec = vec![
                (TokenType::Predicate, "tag"),
                (TokenType::Colon, ":"),
                (TokenType::Arg, "kittens"),
                (TokenType::Or, "or"),
                (TokenType::Predicate, "tag"),
                (TokenType::Colon, ":"),
                (TokenType::Arg, "clouds"),
                (TokenType::And, "and"),
                (TokenType::Predicate, "tag"),
                (TokenType::Colon, ":"),
                (TokenType::Arg, "rain"),
            ];
            verify_success("tag:kittens or tag:clouds and tag:rain", vec);
        }

        #[test]
        fn test_lexer_repeated_negation() {
            let vec = vec![
                (TokenType::Not, "-"),
                (TokenType::Not, "-"),
                (TokenType::Predicate, "tag"),
                (TokenType::Colon, ":"),
                (TokenType::Arg, "kittens"),
                (TokenType::Or, "or"),
                (TokenType::Not, "-"),
                (TokenType::Predicate, "tag"),
                (TokenType::Colon, ":"),
                (TokenType::Arg, "clouds"),
            ];
            verify_success("--tag:kittens or - tag:clouds", vec);
        }

        #[test]
        fn test_lexer_perkeep_search_example() {
            let vec = vec![
                (TokenType::Not, "-"),
                (TokenType::Open, "("),
                (TokenType::Predicate, "after"),
                (TokenType::Colon, ":"),
                (TokenType::Arg, "2010-01-01"),
                (TokenType::Predicate, "before"),
                (TokenType::Colon, ":"),
                (TokenType::Arg, "2010-03-02T12:33:44"),
                (TokenType::Close, ")"),
                (TokenType::Or, "or"),
                (TokenType::Predicate, "loc"),
                (TokenType::Colon, ":"),
                (TokenType::Arg, "Amsterdam"),
            ];
            verify_success(
                "-(after:\"2010-01-01\" before:\"2010-03-02T12:33:44\") or loc:\"Amsterdam\"",
                vec,
            );
        }
    }
}
