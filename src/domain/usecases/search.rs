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
        // Clone the parameters to allow modifying them in-place to make the
        // query and filtering implementation logic simpler.
        let mut params = params.clone();
        // Perform the initial query to get all results, removing whatever
        // criteria was selected so the filtering is straightforward.
        let mut results = self.query_assets(&mut params)?;
        // Filter the results using all of the remaining search criteria.
        results = filter_by_date_range(results, &params);
        results = filter_by_locations(results, &params);
        results = filter_by_filename(results, &params);
        results = filter_by_mimetype(results, &params);
        // Finally, sort the results if so desired.
        sort_results(&mut results, &params);
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
        // All filtering comparisons are case-insensitive for now, so both the
        // input and the index values are lowercased.
        let locations: Vec<String> = params.locations.iter().map(|v| v.to_lowercase()).collect();
        results
            .into_iter()
            .filter(|r| {
                if let Some(row_location) = r.location.as_ref() {
                    let location = row_location.to_lowercase();
                    locations.iter().any(|l| l == &location)
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
fn filter_by_mimetype(results: Vec<SearchResult>, params: &Params) -> Vec<SearchResult> {
    if let Some(p_mimetype) = params.mimetype.as_ref() {
        // All filtering comparisons are case-insensitive for now, so both the
        // input and the index values are lowercased.
        let mimetype = p_mimetype.to_lowercase();
        results
            .into_iter()
            .filter(|r| {
                let lowercase = r.media_type.to_lowercase();
                mimetype == lowercase
            })
            .collect()
    } else {
        results
    }
}

// If a sort was requested, sort the results in-place using an unstable sort
// since it conserves space and the original ordering is not at all important
// (or known for that matter).
fn sort_results(results: &mut Vec<SearchResult>, params: &Params) {
    if let Some(field) = params.sort_field {
        let order = params.sort_order.unwrap_or(SortOrder::Ascending);
        let compare = match field {
            SortField::Date => {
                if order == SortOrder::Ascending {
                    sort_by_date_ascending
                } else {
                    sort_by_date_descending
                }
            }
        };
        results.sort_unstable_by(compare)
    }
}

fn sort_by_date_ascending(a: &SearchResult, b: &SearchResult) -> std::cmp::Ordering {
    a.datetime.cmp(&b.datetime)
}

fn sort_by_date_descending(a: &SearchResult, b: &SearchResult) -> std::cmp::Ordering {
    b.datetime.cmp(&a.datetime)
}

/// Field of the search results on which to sort.
#[derive(Clone, Copy)]
pub enum SortField {
    Date,
}

/// Order by which to sort the search results.
///
/// If not specified in the search paramaters, the default is ascending.
#[derive(Clone, Copy, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Clone, Default)]
pub struct Params {
    pub tags: Vec<String>,
    pub locations: Vec<String>,
    pub filename: Option<String>,
    pub mimetype: Option<String>,
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
    fn test_search_assets_tags_ok() {
        // arrange
        let results = vec![SearchResult {
            asset_id: "cafebabe".to_owned(),
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
            asset_id: "cafebabe".to_owned(),
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
            asset_id: "cafebabe".to_owned(),
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
            asset_id: "cafebabe".to_owned(),
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
            asset_id: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some("Hawaii".to_owned()),
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
            asset_id: "cafebabe".to_owned(),
            filename: "IMG_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some("hawaii".to_owned()),
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
    fn test_search_assets_mimetype_ok() {
        // arrange
        let results = vec![SearchResult {
            asset_id: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/JPEG".to_owned(),
            location: Some("hawaii".to_owned()),
            datetime: Utc::now(),
        }];
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_mimetype()
            .with(eq("imaGE/jpeg"))
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.mimetype = Some("imaGE/jpeg".to_owned());
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
                filename: "IMG_1234.PNG".to_owned(),
                media_type: "IMAGE/PNG".to_owned(),
                location: Some("HAWAII".to_owned()),
                datetime: Utc.ymd(2012, 5, 31).and_hms(21, 10, 11),
            },
            SearchResult {
                asset_id: "babecafe".to_owned(),
                filename: "IMG_2345.GIF".to_owned(),
                media_type: "IMAGE/GIF".to_owned(),
                location: Some("LONDON".to_owned()),
                datetime: Utc.ymd(2013, 5, 31).and_hms(21, 10, 11),
            },
            SearchResult {
                asset_id: "cafed00d".to_owned(),
                filename: "IMG_3456.MOV".to_owned(),
                media_type: "VIDEO/QUICKTIME".to_owned(),
                location: Some("PARIS".to_owned()),
                datetime: Utc.ymd(2014, 5, 31).and_hms(21, 10, 11),
            },
            SearchResult {
                asset_id: "d00dcafe".to_owned(),
                filename: "IMG_4567.JPG".to_owned(),
                media_type: "IMAGE/JPEG".to_owned(),
                location: Some("HAWAII".to_owned()),
                datetime: Utc.ymd(2015, 5, 31).and_hms(21, 10, 11),
            },
            SearchResult {
                asset_id: "deadbeef".to_owned(),
                filename: "IMG_5678.MOV".to_owned(),
                media_type: "VIDEO/QUICKTIME".to_owned(),
                location: Some("LONDON".to_owned()),
                datetime: Utc.ymd(2016, 5, 31).and_hms(21, 10, 11),
            },
            SearchResult {
                asset_id: "cafebeef".to_owned(),
                filename: "IMG_6789.JPG".to_owned(),
                media_type: "IMAGE/JPEG".to_owned(),
                location: Some("PARIS".to_owned()),
                datetime: Utc.ymd(2017, 5, 31).and_hms(21, 10, 11),
            },
            SearchResult {
                asset_id: "deadcafe".to_owned(),
                filename: "IMG_7890.JPG".to_owned(),
                media_type: "IMAGE/JPEG".to_owned(),
                location: Some("YOSEMITE".to_owned()),
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
        assert!(results.iter().any(|l| l.filename == "IMG_2345.GIF"));
        assert!(results.iter().any(|l| l.filename == "IMG_3456.MOV"));
        assert!(results.iter().any(|l| l.filename == "IMG_5678.MOV"));
        assert!(results.iter().any(|l| l.filename == "IMG_6789.JPG"));
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
        assert_eq!(results[0].filename, "IMG_2345.GIF");
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
        assert!(results.iter().any(|l| l.filename == "IMG_3456.MOV"));
        assert!(results.iter().any(|l| l.filename == "IMG_5678.MOV"));
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
        assert!(results.iter().any(|l| l.filename == "IMG_3456.MOV"));
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
        assert!(results.iter().any(|l| l.filename == "IMG_5678.MOV"));
        assert!(results.iter().any(|l| l.filename == "IMG_6789.JPG"));
        assert!(results.iter().any(|l| l.filename == "IMG_7890.JPG"));
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
        assert!(results.iter().any(|l| l.filename == "IMG_1234.PNG"));
        assert!(results.iter().any(|l| l.filename == "IMG_2345.GIF"));
        assert!(results.iter().any(|l| l.filename == "IMG_3456.MOV"));
        assert!(results.iter().any(|l| l.filename == "IMG_4567.JPG"));
    }

    #[test]
    fn test_order_results_ascending_date() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["kitten".to_owned()];
        params.sort_field = Some(SortField::Date);
        params.sort_order = Some(SortOrder::Ascending);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 7);
        assert_eq!(results[0].filename, "IMG_1234.PNG");
        assert_eq!(results[1].filename, "IMG_2345.GIF");
        assert_eq!(results[2].filename, "IMG_3456.MOV");
        assert_eq!(results[3].filename, "IMG_4567.JPG");
        assert_eq!(results[4].filename, "IMG_5678.MOV");
        assert_eq!(results[5].filename, "IMG_6789.JPG");
        assert_eq!(results[6].filename, "IMG_7890.JPG");
    }

    #[test]
    fn test_order_results_descending_date() {
        // arrange
        let results = make_search_results();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = SearchAssets::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.tags = vec!["kitten".to_owned()];
        params.sort_field = Some(SortField::Date);
        params.sort_order = Some(SortOrder::Descending);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 7);
        assert_eq!(results[0].filename, "IMG_7890.JPG");
        assert_eq!(results[1].filename, "IMG_6789.JPG");
        assert_eq!(results[2].filename, "IMG_5678.MOV");
        assert_eq!(results[3].filename, "IMG_4567.JPG");
        assert_eq!(results[4].filename, "IMG_3456.MOV");
        assert_eq!(results[5].filename, "IMG_2345.GIF");
        assert_eq!(results[6].filename, "IMG_1234.PNG");
    }
}
