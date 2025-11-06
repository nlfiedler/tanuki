//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::{SearchParams, SearchResult};
use crate::domain::repositories::{RecordRepository, SearchRepository};
use anyhow::Error;
use hashed_array_tree::HashedArrayTree;

/// Use case to perform queries against the database indices.
pub struct SearchAssets {
    repo: Box<dyn RecordRepository>,
    cache: Box<dyn SearchRepository>,
}

impl SearchAssets {
    pub fn new(repo: Box<dyn RecordRepository>, cache: Box<dyn SearchRepository>) -> Self {
        Self { repo, cache }
    }

    // Perform an initial search of the assets.
    fn query_assets(
        &self,
        params: &mut SearchParams,
    ) -> Result<HashedArrayTree<SearchResult>, Error> {
        // Perform the initial query using one of the criteria. Querying by tags
        // is the first choice since the search results do not contain the tags,
        // so a filter on tags would not be possible.
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
        } else if let Some(media_type) = params.media_type.take() {
            self.repo.query_by_media_type(&media_type)
        } else {
            // did not recognize the query, return nothing
            Ok(HashedArrayTree::<SearchResult>::new())
        }
    }
}

impl super::UseCase<HashedArrayTree<SearchResult>, SearchParams> for SearchAssets {
    fn call(&self, params: SearchParams) -> Result<HashedArrayTree<SearchResult>, Error> {
        // Check if a similar search was performed earlier, ignoring the sorting
        // parameters; the sorting will be performed again as needed.
        let mut results: HashedArrayTree<SearchResult>;
        let cache_key = params.to_string();
        if let Some(cached) = self.cache.get(&cache_key)? {
            results = cached;
        } else {
            let mut params = params.clone();
            // Perform the initial query to get all results, removing whatever
            // criteria was selected so the filtering is straightforward.
            results = self.query_assets(&mut params)?;
            results = filter_by_date_range(results, &params);
            results = filter_by_locations(results, &params);
            results = filter_by_media_type(results, &params);
            self.cache.put(cache_key, results.clone())?;
        }
        super::sort_results(&mut results, params.sort_field, params.sort_order);
        Ok(results)
    }
}

// Filter the search results by date range, if specified.
fn filter_by_date_range(
    results: HashedArrayTree<SearchResult>,
    params: &SearchParams,
) -> HashedArrayTree<SearchResult> {
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
fn filter_by_locations(
    results: HashedArrayTree<SearchResult>,
    params: &SearchParams,
) -> HashedArrayTree<SearchResult> {
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

// Filter the search results by media type, if specified.
fn filter_by_media_type(
    results: HashedArrayTree<SearchResult>,
    params: &SearchParams,
) -> HashedArrayTree<SearchResult> {
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

#[cfg(test)]
mod tests {
    use super::super::UseCase;
    use super::*;
    use crate::domain::entities::{Location, SortField, SortOrder};
    use crate::domain::repositories::{MockRecordRepository, MockSearchRepository};
    use anyhow::anyhow;
    use chrono::prelude::*;
    use hashed_array_tree::hat;
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
    fn test_search_params_format() {
        let params = SearchParams {
            tags: vec![],
            locations: vec![],
            media_type: None,
            before_date: None,
            after_date: None,
            sort_field: None,
            sort_order: None,
        };
        assert_eq!(params.to_string(), "");

        let after = make_date_time(2018, 5, 31, 21, 10, 11);
        let before = make_date_time(2019, 8, 30, 12, 8, 31);
        let params = SearchParams {
            tags: vec!["kittens".into(), "puppies".into()],
            locations: vec!["paris".into()],
            media_type: Some("image/jpeg".into()),
            before_date: Some(before),
            after_date: Some(after),
            sort_field: None,
            sort_order: None,
        };
        assert_eq!(params.to_string(), " tag:kittens tag:puppies loc:paris is:image format:jpeg before:2019-08-30 after:2018-05-31");
    }

    #[test]
    fn test_search_assets_tags_ok() {
        // arrange
        let results = hat![SearchResult {
            asset_id: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some(Location::new("hawaii")),
            datetime: Utc::now(),
        }];
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = SearchAssets::new(Box::new(mock), Box::new(cache));
        let params = SearchParams {
            tags: vec!["assets_tags_ok".to_owned()],
            ..Default::default()
        };
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    #[test]
    fn test_search_assets_tags_err() {
        // arrange
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Err(anyhow!("oh no")));
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().never();
        // act
        let usecase = SearchAssets::new(Box::new(mock), Box::new(cache));
        let params = SearchParams {
            tags: vec!["assets_tags_err".to_owned()],
            ..Default::default()
        };
        let result = usecase.call(params);
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_search_assets_after_ok() {
        // arrange
        let results = hat![SearchResult {
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
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = SearchAssets::new(Box::new(mock), Box::new(cache));
        let params = SearchParams {
            after_date: Some(after),
            ..Default::default()
        };
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    #[test]
    fn test_search_assets_before_ok() {
        // arrange
        let results = hat![SearchResult {
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
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = SearchAssets::new(Box::new(mock), Box::new(cache));
        let params = SearchParams {
            before_date: Some(before),
            ..Default::default()
        };
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    #[test]
    fn test_search_assets_range_ok() {
        // arrange
        let results = hat![SearchResult {
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
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = SearchAssets::new(Box::new(mock), Box::new(cache));
        let params = SearchParams {
            after_date: Some(after),
            before_date: Some(before),
            ..Default::default()
        };
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    #[test]
    fn test_search_assets_locations_ok() {
        // arrange
        let results = hat![SearchResult {
            asset_id: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some(Location::new("hawaii")),
            datetime: Utc::now(),
        }];
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_locations()
            .returning(move |_| Ok(results.clone()));
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = SearchAssets::new(Box::new(mock), Box::new(cache));
        let params = SearchParams {
            locations: vec!["hawaii".to_owned()],
            ..Default::default()
        };
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    #[test]
    fn test_search_assets_media_type_ok() {
        // arrange
        let results = hat![SearchResult {
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
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = SearchAssets::new(Box::new(mock), Box::new(cache));
        let params = SearchParams {
            media_type: Some("imaGE/jpeg".to_owned()),
            ..Default::default()
        };
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    fn make_search_results() -> HashedArrayTree<SearchResult> {
        // make everything uppercase for lettercase testing
        hat![
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
    fn test_filter_results_location_single() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = SearchAssets::new(Box::new(mock), Box::new(cache));
        let params = SearchParams {
            tags: vec!["kitten".to_owned()],
            locations: vec!["london".to_owned()],
            ..Default::default()
        };
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|l| l.filename == "IMG_2345.GIF"));
        assert!(results.iter().any(|l| l.filename == "IMG_5678.MOV"));
    }

    #[test]
    fn test_filter_results_location_multiple() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = SearchAssets::new(Box::new(mock), Box::new(cache));
        let params = SearchParams {
            tags: vec!["kitten".to_owned()],
            locations: vec!["london".to_owned(), "england".into()],
            ..Default::default()
        };
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results.iter().any(|l| l.filename == "IMG_5678.MOV"));
    }

    #[test]
    fn test_filter_results_media_type() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = SearchAssets::new(Box::new(mock), Box::new(cache));
        let params = SearchParams {
            tags: vec!["kitten".to_owned()],
            media_type: Some("video/quicktime".to_owned()),
            ..Default::default()
        };
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|l| l.filename == "IMG_6431.MOV"));
        assert!(results.iter().any(|l| l.filename == "IMG_5678.MOV"));
    }

    #[test]
    fn test_filter_results_date_range() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = SearchAssets::new(Box::new(mock), Box::new(cache));
        let params = SearchParams {
            tags: vec!["kitten".to_owned()],
            after_date: Some(make_date_time(2014, 4, 28, 21, 10, 11)),
            before_date: Some(make_date_time(2017, 4, 28, 21, 10, 11)),
            ..Default::default()
        };
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
    fn test_filter_results_after_date() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = SearchAssets::new(Box::new(mock), Box::new(cache));
        let params = SearchParams {
            tags: vec!["kitten".to_owned()],
            after_date: Some(make_date_time(2016, 4, 28, 21, 10, 11)),
            ..Default::default()
        };
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
    fn test_filter_results_before_date() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = SearchAssets::new(Box::new(mock), Box::new(cache));
        let params = SearchParams {
            tags: vec!["kitten".to_owned()],
            before_date: Some(make_date_time(2016, 4, 28, 21, 10, 11)),
            ..Default::default()
        };
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
    fn test_order_results_ascending_date() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = SearchAssets::new(Box::new(mock), Box::new(cache));
        let params = SearchParams {
            tags: vec!["ascending_date".to_owned()],
            sort_field: Some(SortField::Date),
            sort_order: Some(SortOrder::Ascending),
            ..Default::default()
        };
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
    fn test_order_results_descending_date() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = SearchAssets::new(Box::new(mock), Box::new(cache));
        let params = SearchParams {
            tags: vec!["descending_date".to_owned()],
            sort_field: Some(SortField::Date),
            sort_order: Some(SortOrder::Descending),
            ..Default::default()
        };
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
    fn test_search_cache_sort_by_date() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        // ensure query_by_tags() is called exactly once
        mock.expect_query_by_tags()
            .times(1)
            .returning(move |_| Ok(results.clone()));
        let mut cache = MockSearchRepository::new();
        let mut cache_hit = false;
        cache.expect_get().returning(move |_| {
            if cache_hit {
                Ok(Some(make_search_results()))
            } else {
                cache_hit = true;
                Ok(None)
            }
        });
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = SearchAssets::new(Box::new(mock), Box::new(cache));
        let params = SearchParams {
            tags: vec!["kittens".to_owned()],
            sort_field: Some(SortField::Date),
            sort_order: Some(SortOrder::Descending),
            ..Default::default()
        };
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
        let params = SearchParams {
            tags: vec!["kittens".to_owned()],
            sort_field: Some(SortField::Date),
            sort_order: Some(SortOrder::Ascending),
            ..Default::default()
        };
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
    fn test_order_results_ascending_filename() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = SearchAssets::new(Box::new(mock), Box::new(cache));
        let params = SearchParams {
            tags: vec!["ascending_filename".to_owned()],
            sort_field: Some(SortField::Filename),
            sort_order: Some(SortOrder::Ascending),
            ..Default::default()
        };
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
    fn test_order_results_descending_filename() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = SearchAssets::new(Box::new(mock), Box::new(cache));
        let params = SearchParams {
            tags: vec!["descending_filename".to_owned()],
            sort_field: Some(SortField::Filename),
            sort_order: Some(SortOrder::Descending),
            ..Default::default()
        };
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
    fn test_order_results_ascending_identifier() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = SearchAssets::new(Box::new(mock), Box::new(cache));
        let params = SearchParams {
            tags: vec!["ascending_identifier".to_owned()],
            sort_field: Some(SortField::Identifier),
            sort_order: Some(SortOrder::Ascending),
            ..Default::default()
        };
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
    fn test_order_results_descending_identifier() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = SearchAssets::new(Box::new(mock), Box::new(cache));
        let params = SearchParams {
            tags: vec!["descending_identifier".to_owned()],
            sort_field: Some(SortField::Identifier),
            sort_order: Some(SortOrder::Descending),
            ..Default::default()
        };
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
    fn test_order_results_ascending_media_type() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = SearchAssets::new(Box::new(mock), Box::new(cache));
        let params = SearchParams {
            tags: vec!["ascending_media_type".to_owned()],
            sort_field: Some(SortField::MediaType),
            sort_order: Some(SortOrder::Ascending),
            ..Default::default()
        };
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
    fn test_order_results_descending_media_type() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        let mut cache = MockSearchRepository::new();
        cache.expect_get().returning(|_| Ok(None));
        cache.expect_put().once().returning(|_, _| Ok(()));
        // act
        let usecase = SearchAssets::new(Box::new(mock), Box::new(cache));
        let params = SearchParams {
            tags: vec!["descending_media_type".to_owned()],
            sort_field: Some(SortField::MediaType),
            sort_order: Some(SortOrder::Descending),
            ..Default::default()
        };
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
