//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::{SearchResult, SortField, SortOrder};
use crate::domain::repositories::RecordRepository;
use anyhow::Error;
use chrono::prelude::*;
use std::cmp;
use std::fmt;
use std::num::NonZeroUsize;
use std::sync::{LazyLock, Mutex};

// Average search result entry will be around 128 bytes. In the worst case, if
// search results are 256 bytes and the search yields 10,000 results, the space
// required will be 2.5 MB. As such, maintain a cache of one entry at most.
static LRU_CACHE: LazyLock<Mutex<lru::LruCache<Params, Vec<SearchResult>>>> =
    LazyLock::new(|| Mutex::new(lru::LruCache::new(NonZeroUsize::new(1).unwrap())));

/// Use case to perform queries against the database indices.
pub struct SearchAssets {
    repo: Box<dyn RecordRepository>,
}

impl SearchAssets {
    pub fn new(repo: Box<dyn RecordRepository>) -> Self {
        Self { repo }
    }

    // Perform an initial search of the assets.
    fn query_assets(&self, params: &mut Params) -> Result<Vec<SearchResult>, Error> {
        //
        // Perform the initial query using one of the criteria. The tags is the
        // first choice since the secondary index does not contain any tags, so
        // a filter on tags is not possible. What's more, the tags query is more
        // sophisticated (matches assets that have _all_ of the keys, not just
        // one) and as such filtering would not make sense.
        //
        if !params.tags.is_empty() {
            let tags = params.tags.drain(..).collect();
            self.repo.query_by_tags(tags)
        } else if params.after_date.is_some() && params.before_date.is_some() {
            let after = params.after_date.take().unwrap();
            let before = params.before_date.take().unwrap();
            self.repo.query_date_range(after, before)
        } else if params.before_date.is_some() {
            let before = params.before_date.take().unwrap();
            self.repo.query_before_date(before)
        } else if params.after_date.is_some() {
            let after = params.after_date.take().unwrap();
            self.repo.query_after_date(after)
        } else if !params.locations.is_empty() {
            let locations = params.locations.drain(..).collect();
            self.repo.query_by_locations(locations)
        } else if let Some(filename) = params.filename.take() {
            self.repo.query_by_filename(&filename)
        } else if let Some(media_type) = params.media_type.take() {
            self.repo.query_by_media_type(&media_type)
        } else {
            // did not recognize the query, return nothing
            Ok(vec![])
        }
    }
}

impl super::UseCase<Vec<SearchResult>, Params> for SearchAssets {
    fn call(&self, params: Params) -> Result<Vec<SearchResult>, Error> {
        // Check if a similar search was performed earlier, ignoring the sorting
        // parameters; the sorting will be performed again as needed.
        let mut cache = LRU_CACHE.lock().unwrap();
        let mut results: Vec<SearchResult>;
        if let Some(cached) = cache.get(&params) {
            results = cached.to_owned();
        } else {
            let original_params = params.clone();
            let mut params = params.clone();
            // Perform the initial query to get all results, removing whatever
            // criteria was selected so the filtering is straightforward.
            results = self.query_assets(&mut params)?;
            results = filter_by_date_range(results, &params);
            results = filter_by_locations(results, &params);
            results = filter_by_filename(results, &params);
            results = filter_by_media_type(results, &params);
            cache.put(original_params, results.clone());
        }
        super::sort_results(&mut results, params.sort_field, params.sort_order);
        Ok(results)
    }
}

// Filter the search results by date range, if specified.
fn filter_by_date_range(results: Vec<SearchResult>, params: &Params) -> Vec<SearchResult> {
    if params.after_date.is_some() && params.before_date.is_some() {
        let a = params.after_date.unwrap();
        let b = params.before_date.unwrap();
        results
            .into_iter()
            .filter(|r| r.datetime > a && r.datetime < b)
            .collect()
    } else if params.after_date.is_some() {
        let a = params.after_date.unwrap();
        results.into_iter().filter(|r| r.datetime > a).collect()
    } else if params.before_date.is_some() {
        let b = params.before_date.unwrap();
        results.into_iter().filter(|r| r.datetime < b).collect()
    } else {
        results
    }
}

// Filter the search results by location(s), if specified.
//
// Matches a result if it contains all of the specified location values. This
// means that a search for "paris, texas" will turn up results that have both
// "paris" and "texas" as part of the location entry, and not simply return
// results that contain either "paris" or "texas".
fn filter_by_locations(results: Vec<SearchResult>, params: &Params) -> Vec<SearchResult> {
    if params.locations.is_empty() {
        results
    } else {
        // All filtering comparisons are case-insensitive for now, so both the
        // input and the index values are lowercased.
        let locations: Vec<String> = params.locations.iter().map(|v| v.to_lowercase()).collect();
        results
            .into_iter()
            .filter(|r| {
                if let Some(location) = r.location.as_ref() {
                    locations.iter().all(|l| location.partial_match(l))
                } else {
                    false
                }
            })
            .collect()
    }
}

// Filter the search results by file name, if specified.
fn filter_by_filename(results: Vec<SearchResult>, params: &Params) -> Vec<SearchResult> {
    if let Some(p_filename) = params.filename.as_ref() {
        // All filtering comparisons are case-insensitive for now, so both the
        // input and the index values are lowercased.
        let filename = p_filename.to_lowercase();
        results
            .into_iter()
            .filter(|r| {
                let lowercase = r.filename.to_lowercase();
                filename == lowercase
            })
            .collect()
    } else {
        results
    }
}

// Filter the search results by media type, if specified.
fn filter_by_media_type(results: Vec<SearchResult>, params: &Params) -> Vec<SearchResult> {
    if let Some(p_media_type) = params.media_type.as_ref() {
        // All filtering comparisons are case-insensitive for now, so both the
        // input and the index values are lowercased.
        let media_type = p_media_type.to_lowercase();
        results
            .into_iter()
            .filter(|r| {
                let lowercase = r.media_type.to_lowercase();
                media_type == lowercase
            })
            .collect()
    } else {
        results
    }
}

#[derive(Clone, Default)]
pub struct Params {
    pub tags: Vec<String>,
    pub locations: Vec<String>,
    pub filename: Option<String>,
    pub media_type: Option<String>,
    pub before_date: Option<DateTime<Utc>>,
    pub after_date: Option<DateTime<Utc>>,
    pub sort_field: Option<SortField>,
    pub sort_order: Option<SortOrder>,
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Params(tags: {})", self.tags.len())
    }
}

impl cmp::PartialEq for Params {
    fn eq(&self, other: &Self) -> bool {
        // ignore sorting when comparing two sets of parameters
        self.tags == other.tags
            && self.locations == other.locations
            && self.filename == other.filename
            && self.media_type == other.media_type
            && self.before_date == other.before_date
            && self.after_date == other.after_date
    }
}

impl std::hash::Hash for Params {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // ignore sorting when hashing a set of parameters
        self.tags.hash(state);
        self.locations.hash(state);
        self.filename.hash(state);
        self.media_type.hash(state);
        self.before_date.hash(state);
        self.after_date.hash(state);
    }
}

impl cmp::Eq for Params {}

#[cfg(test)]
mod tests {
    use super::super::UseCase;
    use super::*;
    use crate::domain::entities::Location;
    use crate::domain::repositories::MockRecordRepository;
    use anyhow::anyhow;
    use mockall::predicate::*;

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
    #[serial_test::serial]
    fn test_search_cache_hashing() {
        // Ensure the lru hashing is working as expected, such that parameters
        // that change only in terms of sorting will still retrieve the cached
        // set of results.
        let mut cache = lru::LruCache::new(NonZeroUsize::new(2).unwrap());
        let mut params: Params = Default::default();
        params.tags = vec!["kitten".to_owned()];
        params.sort_field = Some(SortField::Date);
        params.sort_order = Some(SortOrder::Ascending);
        let results: Vec<String> = vec![
            "honey".into(),
            "marmelade".into(),
            "marmite".into(),
            "mustard".into(),
            "sugar".into(),
            "maple syrup".into(),
        ];
        cache.put(params, results);

        let mut params: Params = Default::default();
        params.tags = vec!["kitten".to_owned()];
        params.sort_field = Some(SortField::Date);
        params.sort_order = Some(SortOrder::Descending);
        assert!(cache.contains(&params));

        // different parameters are _not_ in the cache
        let mut params: Params = Default::default();
        params.tags = vec!["puppy".to_owned()];
        params.sort_field = Some(SortField::Date);
        params.sort_order = Some(SortOrder::Ascending);
        assert_eq!(cache.contains(&params), false);
    }

    #[test]
    #[serial_test::serial]
    fn test_search_assets_tags_ok() {
        // arrange
        let results = vec![SearchResult {
            asset_id: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some(Location::new("hawaii")),
            datetime: Utc::now(),
        }];
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["assets_tags_ok".to_owned()];
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    #[test]
    #[serial_test::serial]
    fn test_search_assets_tags_err() {
        // arrange
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Err(anyhow!("oh no")));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["assets_tags_err".to_owned()];
        let result = usecase.call(params);
        // assert
        assert!(result.is_err());
    }

    #[test]
    #[serial_test::serial]
    fn test_search_assets_after_ok() {
        // arrange
        let results = vec![SearchResult {
            asset_id: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some(Location::new("hawaii")),
            datetime: Utc::now(),
        }];
        let after = make_date_time(2018, 5, 31, 21, 10, 11);
        let mut mock = MockRecordRepository::new();
        mock.expect_query_after_date()
            .with(eq(after))
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.after_date = Some(after);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    #[test]
    #[serial_test::serial]
    fn test_search_assets_before_ok() {
        // arrange
        let results = vec![SearchResult {
            asset_id: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some(Location::new("hawaii")),
            datetime: Utc::now(),
        }];
        let before = make_date_time(2018, 5, 31, 21, 10, 11);
        let mut mock = MockRecordRepository::new();
        mock.expect_query_before_date()
            .with(eq(before))
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.before_date = Some(before);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    #[test]
    #[serial_test::serial]
    fn test_search_assets_range_ok() {
        // arrange
        let results = vec![SearchResult {
            asset_id: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some(Location::new("hawaii")),
            datetime: Utc::now(),
        }];
        let after = make_date_time(2018, 1, 31, 21, 10, 11);
        let before = make_date_time(2018, 5, 31, 21, 10, 11);
        let mut mock = MockRecordRepository::new();
        mock.expect_query_date_range()
            .with(eq(after), eq(before))
            .returning(move |_, _| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.after_date = Some(after);
        params.before_date = Some(before);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    #[test]
    #[serial_test::serial]
    fn test_search_assets_locations_ok() {
        // arrange
        let results = vec![SearchResult {
            asset_id: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some(Location::new("hawaii")),
            datetime: Utc::now(),
        }];
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_locations()
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.locations = vec!["hawaii".to_owned()];
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    #[test]
    #[serial_test::serial]
    fn test_search_assets_filename_ok() {
        // arrange
        let results = vec![SearchResult {
            asset_id: "cafebabe".to_owned(),
            filename: "IMG_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some(Location::new("hawaii")),
            datetime: Utc::now(),
        }];
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_filename()
            .with(eq("Img_1234.jpg"))
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.filename = Some("Img_1234.jpg".to_owned());
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "IMG_1234.jpg");
    }

    #[test]
    #[serial_test::serial]
    fn test_search_assets_media_type_ok() {
        // arrange
        let results = vec![SearchResult {
            asset_id: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/JPEG".to_owned(),
            location: Some(Location::new("hawaii")),
            datetime: Utc::now(),
        }];
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_media_type()
            .with(eq("imaGE/jpeg"))
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.media_type = Some("imaGE/jpeg".to_owned());
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    fn make_search_results() -> Vec<SearchResult> {
        // make everything uppercase for lettercase testing
        vec![
            SearchResult {
                asset_id: "cafebabe".to_owned(),
                filename: "IMG_2431.PNG".to_owned(),
                media_type: "IMAGE/PNG".to_owned(),
                location: Some(Location::new("hawaii")),
                datetime: make_date_time(2012, 5, 31, 21, 10, 11),
            },
            SearchResult {
                asset_id: "babecafe".to_owned(),
                filename: "IMG_2345.GIF".to_owned(),
                media_type: "IMAGE/GIF".to_owned(),
                location: Some(Location::new("london")),
                datetime: make_date_time(2013, 5, 31, 21, 10, 11),
            },
            SearchResult {
                asset_id: "cafed00d".to_owned(),
                filename: "IMG_6431.MOV".to_owned(),
                media_type: "VIDEO/QUICKTIME".to_owned(),
                location: Some(Location::new("paris")),
                datetime: make_date_time(2014, 5, 31, 21, 10, 11),
            },
            SearchResult {
                asset_id: "d00dcafe".to_owned(),
                filename: "IMG_4567.JPG".to_owned(),
                media_type: "IMAGE/JPEG".to_owned(),
                location: Some(Location::new("hawaii")),
                datetime: make_date_time(2015, 5, 31, 21, 10, 11),
            },
            SearchResult {
                asset_id: "deadbeef".to_owned(),
                filename: "IMG_5678.MOV".to_owned(),
                media_type: "VIDEO/QUICKTIME".to_owned(),
                location: Some(Location::with_parts("", "london", "england")),
                datetime: make_date_time(2016, 5, 31, 21, 10, 11),
            },
            SearchResult {
                asset_id: "cafebeef".to_owned(),
                filename: "IMG_6789.JPG".to_owned(),
                media_type: "IMAGE/JPEG".to_owned(),
                location: Some(Location::new("paris")),
                datetime: make_date_time(2017, 5, 31, 21, 10, 11),
            },
            SearchResult {
                asset_id: "deadcafe".to_owned(),
                filename: "IMG_3142.JPG".to_owned(),
                media_type: "IMAGE/JPEG".to_owned(),
                location: Some(Location::new("yosemite")),
                datetime: make_date_time(2018, 5, 31, 21, 10, 11),
            },
        ]
    }

    #[test]
    #[serial_test::serial]
    fn test_filter_results_location_single() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["kitten".to_owned()];
        params.locations = vec!["london".to_owned()];
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|l| l.filename == "IMG_2345.GIF"));
        assert!(results.iter().any(|l| l.filename == "IMG_5678.MOV"));
    }

    #[test]
    #[serial_test::serial]
    fn test_filter_results_location_multiple() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["kitten".to_owned()];
        params.locations = vec!["london".to_owned(), "england".into()];
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results.iter().any(|l| l.filename == "IMG_5678.MOV"));
    }

    #[test]
    #[serial_test::serial]
    fn test_filter_results_filename() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["kitten".to_owned()];
        params.filename = Some("img_2345.gif".to_owned());
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "IMG_2345.GIF");
    }

    #[test]
    #[serial_test::serial]
    fn test_filter_results_media_type() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["kitten".to_owned()];
        params.media_type = Some("video/quicktime".to_owned());
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|l| l.filename == "IMG_6431.MOV"));
        assert!(results.iter().any(|l| l.filename == "IMG_5678.MOV"));
    }

    #[test]
    #[serial_test::serial]
    fn test_filter_results_date_range() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["kitten".to_owned()];
        params.after_date = Some(make_date_time(2014, 4, 28, 21, 10, 11));
        params.before_date = Some(make_date_time(2017, 4, 28, 21, 10, 11));
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 3);
        assert!(results.iter().any(|l| l.filename == "IMG_6431.MOV"));
        assert!(results.iter().any(|l| l.filename == "IMG_4567.JPG"));
        assert!(results.iter().any(|l| l.filename == "IMG_5678.MOV"));
    }

    #[test]
    #[serial_test::serial]
    fn test_filter_results_after_date() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["kitten".to_owned()];
        params.after_date = Some(make_date_time(2016, 4, 28, 21, 10, 11));
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 3);
        assert!(results.iter().any(|l| l.filename == "IMG_5678.MOV"));
        assert!(results.iter().any(|l| l.filename == "IMG_6789.JPG"));
        assert!(results.iter().any(|l| l.filename == "IMG_3142.JPG"));
    }

    #[test]
    #[serial_test::serial]
    fn test_filter_results_before_date() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["kitten".to_owned()];
        params.before_date = Some(make_date_time(2016, 4, 28, 21, 10, 11));
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 4);
        assert!(results.iter().any(|l| l.filename == "IMG_2431.PNG"));
        assert!(results.iter().any(|l| l.filename == "IMG_2345.GIF"));
        assert!(results.iter().any(|l| l.filename == "IMG_6431.MOV"));
        assert!(results.iter().any(|l| l.filename == "IMG_4567.JPG"));
    }

    #[test]
    #[serial_test::serial]
    fn test_order_results_ascending_date() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["ascending_date".to_owned()];
        params.sort_field = Some(SortField::Date);
        params.sort_order = Some(SortOrder::Ascending);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 7);
        assert_eq!(results[0].filename, "IMG_2431.PNG");
        assert_eq!(results[1].filename, "IMG_2345.GIF");
        assert_eq!(results[2].filename, "IMG_6431.MOV");
        assert_eq!(results[3].filename, "IMG_4567.JPG");
        assert_eq!(results[4].filename, "IMG_5678.MOV");
        assert_eq!(results[5].filename, "IMG_6789.JPG");
        assert_eq!(results[6].filename, "IMG_3142.JPG");
    }

    #[test]
    #[serial_test::serial]
    fn test_order_results_descending_date() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["descending_date".to_owned()];
        params.sort_field = Some(SortField::Date);
        params.sort_order = Some(SortOrder::Descending);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 7);
        assert_eq!(results[0].filename, "IMG_3142.JPG");
        assert_eq!(results[1].filename, "IMG_6789.JPG");
        assert_eq!(results[2].filename, "IMG_5678.MOV");
        assert_eq!(results[3].filename, "IMG_4567.JPG");
        assert_eq!(results[4].filename, "IMG_6431.MOV");
        assert_eq!(results[5].filename, "IMG_2345.GIF");
        assert_eq!(results[6].filename, "IMG_2431.PNG");
    }

    #[test]
    #[serial_test::serial]
    fn test_search_cache_sort_by_date() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        // ensure query_by_tags() is called exactly once
        mock.expect_query_by_tags()
            .times(1)
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["kittens".to_owned()];
        params.sort_field = Some(SortField::Date);
        params.sort_order = Some(SortOrder::Descending);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 7);
        assert_eq!(results[0].filename, "IMG_3142.JPG");
        assert_eq!(results[1].filename, "IMG_6789.JPG");
        assert_eq!(results[2].filename, "IMG_5678.MOV");
        assert_eq!(results[3].filename, "IMG_4567.JPG");
        assert_eq!(results[4].filename, "IMG_6431.MOV");
        assert_eq!(results[5].filename, "IMG_2345.GIF");
        assert_eq!(results[6].filename, "IMG_2431.PNG");

        // act (same search but different sort order, should hit the cache and
        // yet sort the results accordingly)
        let mut params: Params = Default::default();
        params.tags = vec!["kittens".to_owned()];
        params.sort_field = Some(SortField::Date);
        params.sort_order = Some(SortOrder::Ascending);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 7);
        assert_eq!(results[0].filename, "IMG_2431.PNG");
        assert_eq!(results[1].filename, "IMG_2345.GIF");
        assert_eq!(results[2].filename, "IMG_6431.MOV");
        assert_eq!(results[3].filename, "IMG_4567.JPG");
        assert_eq!(results[4].filename, "IMG_5678.MOV");
        assert_eq!(results[5].filename, "IMG_6789.JPG");
        assert_eq!(results[6].filename, "IMG_3142.JPG");
    }

    #[test]
    #[serial_test::serial]
    fn test_order_results_ascending_filename() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["ascending_filename".to_owned()];
        params.sort_field = Some(SortField::Filename);
        params.sort_order = Some(SortOrder::Ascending);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 7);
        assert_eq!(results[0].filename, "IMG_2345.GIF");
        assert_eq!(results[1].filename, "IMG_2431.PNG");
        assert_eq!(results[2].filename, "IMG_3142.JPG");
        assert_eq!(results[3].filename, "IMG_4567.JPG");
        assert_eq!(results[4].filename, "IMG_5678.MOV");
        assert_eq!(results[5].filename, "IMG_6431.MOV");
        assert_eq!(results[6].filename, "IMG_6789.JPG");
    }

    #[test]
    #[serial_test::serial]
    fn test_order_results_descending_filename() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["descending_filename".to_owned()];
        params.sort_field = Some(SortField::Filename);
        params.sort_order = Some(SortOrder::Descending);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 7);
        assert_eq!(results[0].filename, "IMG_6789.JPG");
        assert_eq!(results[1].filename, "IMG_6431.MOV");
        assert_eq!(results[2].filename, "IMG_5678.MOV");
        assert_eq!(results[3].filename, "IMG_4567.JPG");
        assert_eq!(results[4].filename, "IMG_3142.JPG");
        assert_eq!(results[5].filename, "IMG_2431.PNG");
        assert_eq!(results[6].filename, "IMG_2345.GIF");
    }

    #[test]
    #[serial_test::serial]
    fn test_order_results_ascending_identifier() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["ascending_identifier".to_owned()];
        params.sort_field = Some(SortField::Identifier);
        params.sort_order = Some(SortOrder::Ascending);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 7);
        assert_eq!(results[0].asset_id, "babecafe");
        assert_eq!(results[1].asset_id, "cafebabe");
        assert_eq!(results[2].asset_id, "cafebeef");
        assert_eq!(results[3].asset_id, "cafed00d");
        assert_eq!(results[4].asset_id, "d00dcafe");
        assert_eq!(results[5].asset_id, "deadbeef");
        assert_eq!(results[6].asset_id, "deadcafe");
    }

    #[test]
    #[serial_test::serial]
    fn test_order_results_descending_identifier() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["descending_identifier".to_owned()];
        params.sort_field = Some(SortField::Identifier);
        params.sort_order = Some(SortOrder::Descending);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 7);
        assert_eq!(results[0].asset_id, "deadcafe");
        assert_eq!(results[1].asset_id, "deadbeef");
        assert_eq!(results[2].asset_id, "d00dcafe");
        assert_eq!(results[3].asset_id, "cafed00d");
        assert_eq!(results[4].asset_id, "cafebeef");
        assert_eq!(results[5].asset_id, "cafebabe");
        assert_eq!(results[6].asset_id, "babecafe");
    }

    #[test]
    #[serial_test::serial]
    fn test_order_results_ascending_media_type() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["ascending_media_type".to_owned()];
        params.sort_field = Some(SortField::MediaType);
        params.sort_order = Some(SortOrder::Ascending);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 7);
        assert_eq!(results[0].media_type, "IMAGE/GIF");
        assert_eq!(results[1].media_type, "IMAGE/JPEG");
        assert_eq!(results[2].media_type, "IMAGE/JPEG");
        assert_eq!(results[3].media_type, "IMAGE/JPEG");
        assert_eq!(results[4].media_type, "IMAGE/PNG");
        assert_eq!(results[5].media_type, "VIDEO/QUICKTIME");
        assert_eq!(results[6].media_type, "VIDEO/QUICKTIME");
    }

    #[test]
    #[serial_test::serial]
    fn test_order_results_descending_media_type() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["descending_media_type".to_owned()];
        params.sort_field = Some(SortField::MediaType);
        params.sort_order = Some(SortOrder::Descending);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 7);
        assert_eq!(results[0].media_type, "VIDEO/QUICKTIME");
        assert_eq!(results[1].media_type, "VIDEO/QUICKTIME");
        assert_eq!(results[2].media_type, "IMAGE/PNG");
        assert_eq!(results[3].media_type, "IMAGE/JPEG");
        assert_eq!(results[4].media_type, "IMAGE/JPEG");
        assert_eq!(results[5].media_type, "IMAGE/JPEG");
        assert_eq!(results[6].media_type, "IMAGE/GIF");
    }
}
