//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::domain::entities::SearchResult;
use crate::domain::repositories::RecordRepository;
use chrono::prelude::*;
use failure::Error;
use std::cmp;
use std::fmt;

/// Use case to perform complex queries on the asset database.
pub struct SearchAssets {
    repo: Box<dyn RecordRepository>,
}

impl SearchAssets {
    pub fn new(repo: Box<dyn RecordRepository>) -> Self {
        Self { repo }
    }

    // Perform an initial search of the assets.
    fn query_assets(&self, params: &mut Params) -> Result<Vec<SearchResult>, Error> {
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
        } else if let Some(mimetype) = params.mimetype.take() {
            self.repo.query_by_mimetype(&mimetype)
        } else {
            // did not recognize the query, return nothing
            Ok(vec![])
        }
    }
}

impl super::UseCase<Vec<SearchResult>, Params> for SearchAssets {
    fn call(&self, params: Params) -> Result<Vec<SearchResult>, Error> {
        //
        // Copy and lowercase the parameters to match the values in the index.
        // Additionally, the parameters are modified in-place to make the query
        // and filtering implementation logic simpler.
        //
        let mut params = params.lowercase_and_dedup();
        //
        // Start by performing the query on one of the criteria. The tags is the
        // first choice since the secondary index does not contain any tags, so
        // a filter on tags is not possible. What's more, the tags query is more
        // sophisticated (matches assets that have _all_ of the keys, not just
        // one) and as such filtering would not make sense.
        //
        let mut results = self.query_assets(&mut params)?;
        //
        // Once the primary query has been performed, filter the results using
        // all of the remaining search criteria.
        //
        results = filter_by_date_range(results, &params);
        results = filter_by_locations(results, &params);
        results = filter_by_filename(results, &params);
        results = filter_by_mimetype(results, &params);
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
// Matches a result if it contains any of the specified locations.
fn filter_by_locations(results: Vec<SearchResult>, params: &Params) -> Vec<SearchResult> {
    if params.locations.is_empty() {
        results
    } else {
        results
            .into_iter()
            .filter(|r| {
                // index values are not in lowercase
                let location = r
                    .location
                    .as_ref()
                    .map_or("".to_owned(), |s| s.to_lowercase());
                params.locations.iter().any(|l| l == &location)
            })
            .collect()
    }
}

// Filter the search results by file name, if specified.
fn filter_by_filename(results: Vec<SearchResult>, params: &Params) -> Vec<SearchResult> {
    if let Some(filename) = params.filename.as_ref() {
        results
            .into_iter()
            .filter(|r| {
                // index values are not in lowercase
                let lowercase = r.filename.to_lowercase();
                filename == &lowercase
            })
            .collect()
    } else {
        results
    }
}

// Filter the search results by media type, if specified.
fn filter_by_mimetype(results: Vec<SearchResult>, params: &Params) -> Vec<SearchResult> {
    if let Some(mimetype) = params.mimetype.as_ref() {
        results
            .into_iter()
            .filter(|r| {
                // index values are not in lowercase
                let lowercase = r.media_type.to_lowercase();
                mimetype == &lowercase
            })
            .collect()
    } else {
        results
    }
}

#[derive(Default)]
pub struct Params {
    tags: Vec<String>,
    locations: Vec<String>,
    filename: Option<String>,
    mimetype: Option<String>,
    before_date: Option<DateTime<Utc>>,
    after_date: Option<DateTime<Utc>>,
}

impl Params {
    /// Lowercase all string inputs, deduplicate tags, locations.
    fn lowercase_and_dedup(&self) -> Self {
        let mut tags: Vec<String> = self.tags.iter().map(|v| v.to_lowercase()).collect();
        tags.sort();
        tags.dedup();
        let mut locations: Vec<String> = self.locations.iter().map(|v| v.to_lowercase()).collect();
        locations.sort();
        locations.dedup();
        Self {
            tags,
            locations,
            filename: self.filename.as_ref().map(|v| v.to_lowercase()),
            mimetype: self.mimetype.as_ref().map(|v| v.to_lowercase()),
            before_date: self.before_date,
            after_date: self.after_date,
        }
    }
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Params(tags: {})", self.tags.len())
    }
}

impl cmp::PartialEq for Params {
    fn eq(&self, other: &Self) -> bool {
        self.tags == other.tags
    }
}

impl cmp::Eq for Params {}

#[cfg(test)]
mod tests {
    use super::super::UseCase;
    use super::*;
    use crate::domain::repositories::MockRecordRepository;
    use failure::err_msg;
    use mockall::predicate::*;

    #[test]
    fn test_params_lowercase_and_dedup() {
        // arrange
        let mut params: Params = Default::default();
        params.tags = vec![
            "puppy".to_owned(),
            "KITTEN".to_owned(),
            "Kitten".to_owned(),
            "kitten".to_owned(),
        ];
        params.locations = vec![
            "paris".to_owned(),
            "HAWAII".to_owned(),
            "Hawaii".to_owned(),
            "hawaii".to_owned(),
        ];
        params.filename = Some("IMG_1234.jpg".to_owned());
        params.mimetype = Some("Image/JPEG".to_owned());
        // act
        let actual = params.lowercase_and_dedup();
        // assert
        assert_eq!(actual.filename.unwrap(), "img_1234.jpg");
        assert_eq!(actual.mimetype.unwrap(), "image/jpeg");
        assert_eq!(actual.tags.len(), 2);
        assert_eq!(actual.tags[0], "kitten");
        assert_eq!(actual.tags[1], "puppy");
        assert_eq!(actual.locations.len(), 2);
        assert_eq!(actual.locations[0], "hawaii");
        assert_eq!(actual.locations[1], "paris");
    }

    #[test]
    fn test_search_assets_tags_ok() {
        // arrange
        let results = vec![SearchResult {
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some("hawaii".to_owned()),
            datetime: Utc::now(),
        }];
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["kitten".to_owned()];
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    #[test]
    fn test_search_assets_err() {
        // arrange
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Err(err_msg("oh no")));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["kitten".to_owned()];
        let result = usecase.call(params);
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_search_assets_after_ok() {
        // arrange
        let results = vec![SearchResult {
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some("hawaii".to_owned()),
            datetime: Utc::now(),
        }];
        let after = Utc.ymd(2018, 5, 31).and_hms(21, 10, 11);
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
    fn test_search_assets_before_ok() {
        // arrange
        let results = vec![SearchResult {
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some("hawaii".to_owned()),
            datetime: Utc::now(),
        }];
        let before = Utc.ymd(2018, 5, 31).and_hms(21, 10, 11);
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
    fn test_search_assets_range_ok() {
        // arrange
        let results = vec![SearchResult {
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some("hawaii".to_owned()),
            datetime: Utc::now(),
        }];
        let after = Utc.ymd(2018, 1, 31).and_hms(21, 10, 11);
        let before = Utc.ymd(2018, 5, 13).and_hms(21, 10, 11);
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
    fn test_search_assets_locations_ok() {
        // arrange
        let results = vec![SearchResult {
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some("hawaii".to_owned()),
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
    fn test_search_assets_filename_ok() {
        // arrange
        let results = vec![SearchResult {
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some("hawaii".to_owned()),
            datetime: Utc::now(),
        }];
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_filename()
            .with(eq("img_1234.jpg"))
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.filename = Some("img_1234.jpg".to_owned());
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    #[test]
    fn test_search_assets_mimetype_ok() {
        // arrange
        let results = vec![SearchResult {
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some("hawaii".to_owned()),
            datetime: Utc::now(),
        }];
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_mimetype()
            .with(eq("image/jpeg"))
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.mimetype = Some("image/jpeg".to_owned());
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    fn make_search_results() -> Vec<SearchResult> {
        vec![
            SearchResult {
                filename: "img_1234.png".to_owned(),
                media_type: "image/png".to_owned(),
                location: Some("hawaii".to_owned()),
                datetime: Utc.ymd(2012, 5, 31).and_hms(21, 10, 11),
            },
            SearchResult {
                filename: "img_2345.gif".to_owned(),
                media_type: "image/gif".to_owned(),
                location: Some("london".to_owned()),
                datetime: Utc.ymd(2013, 5, 31).and_hms(21, 10, 11),
            },
            SearchResult {
                filename: "img_3456.mov".to_owned(),
                media_type: "video/quicktime".to_owned(),
                location: Some("paris".to_owned()),
                datetime: Utc.ymd(2014, 5, 31).and_hms(21, 10, 11),
            },
            SearchResult {
                filename: "img_4567.jpg".to_owned(),
                media_type: "image/jpeg".to_owned(),
                location: Some("hawaii".to_owned()),
                datetime: Utc.ymd(2015, 5, 31).and_hms(21, 10, 11),
            },
            SearchResult {
                filename: "img_5678.mov".to_owned(),
                media_type: "video/quicktime".to_owned(),
                location: Some("london".to_owned()),
                datetime: Utc.ymd(2016, 5, 31).and_hms(21, 10, 11),
            },
            SearchResult {
                filename: "img_6789.jpg".to_owned(),
                media_type: "image/jpeg".to_owned(),
                location: Some("paris".to_owned()),
                datetime: Utc.ymd(2017, 5, 31).and_hms(21, 10, 11),
            },
            SearchResult {
                filename: "img_7890.jpg".to_owned(),
                media_type: "image/jpeg".to_owned(),
                location: Some("yosemite".to_owned()),
                datetime: Utc.ymd(2018, 5, 31).and_hms(21, 10, 11),
            },
        ]
    }

    #[test]
    fn test_filter_results_location() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["kitten".to_owned()];
        params.locations = vec!["london".to_owned(), "paris".to_owned()];
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 4);
        assert!(results.iter().any(|l| l.filename == "img_2345.gif"));
        assert!(results.iter().any(|l| l.filename == "img_3456.mov"));
        assert!(results.iter().any(|l| l.filename == "img_5678.mov"));
        assert!(results.iter().any(|l| l.filename == "img_6789.jpg"));
    }

    #[test]
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
        assert_eq!(results[0].filename, "img_2345.gif");
    }

    #[test]
    fn test_filter_results_mimetype() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["kitten".to_owned()];
        params.mimetype = Some("video/quicktime".to_owned());
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|l| l.filename == "img_3456.mov"));
        assert!(results.iter().any(|l| l.filename == "img_5678.mov"));
    }

    #[test]
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
        params.after_date = Some(Utc.ymd(2014, 4, 28).and_hms(21, 10, 11));
        params.before_date = Some(Utc.ymd(2017, 4, 28).and_hms(21, 10, 11));
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 3);
        assert!(results.iter().any(|l| l.filename == "img_3456.mov"));
        assert!(results.iter().any(|l| l.filename == "img_4567.jpg"));
        assert!(results.iter().any(|l| l.filename == "img_5678.mov"));
    }

    #[test]
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
        params.after_date = Some(Utc.ymd(2016, 4, 28).and_hms(21, 10, 11));
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 3);
        assert!(results.iter().any(|l| l.filename == "img_5678.mov"));
        assert!(results.iter().any(|l| l.filename == "img_6789.jpg"));
        assert!(results.iter().any(|l| l.filename == "img_7890.jpg"));
    }

    #[test]
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
        params.before_date = Some(Utc.ymd(2016, 4, 28).and_hms(21, 10, 11));
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 4);
        assert!(results.iter().any(|l| l.filename == "img_1234.png"));
        assert!(results.iter().any(|l| l.filename == "img_2345.gif"));
        assert!(results.iter().any(|l| l.filename == "img_3456.mov"));
        assert!(results.iter().any(|l| l.filename == "img_4567.jpg"));
    }
}
