//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::domain::entities::SearchResult;
use crate::domain::repositories::RecordRepository;
use chrono::prelude::*;
use failure::Error;
use std::cmp;
use std::fmt;

pub struct SearchAssets {
    repo: Box<dyn RecordRepository>,
}

impl SearchAssets {
    pub fn new(repo: Box<dyn RecordRepository>) -> Self {
        Self { repo }
    }
}

impl super::UseCase<Vec<SearchResult>, Params> for SearchAssets {
    fn call(&self, params: Params) -> Result<Vec<SearchResult>, Error> {
        if !params.tags.is_empty() {
            let half_owned: Vec<_> = params.tags.iter().map(String::as_str).collect();
            self.repo.query_by_tags(&half_owned)
        } else if params.after_date.is_some() && params.before_date.is_some() {
            let after = params.after_date.unwrap();
            let before = params.before_date.unwrap();
            self.repo.query_date_range(after, before)
        } else if params.before_date.is_some() {
            let before = params.before_date.unwrap();
            self.repo.query_before_date(before)
        } else if params.after_date.is_some() {
            let after = params.after_date.unwrap();
            self.repo.query_after_date(after)
        } else if !params.locations.is_empty() {
            let half_owned: Vec<_> = params.locations.iter().map(String::as_str).collect();
            self.repo.query_by_locations(&half_owned)
        } else if let Some(filename) = params.filename {
            self.repo.query_by_filename(&filename)
        } else if let Some(mimetype) = params.mimetype {
            self.repo.query_by_mimetype(&mimetype)
        } else {
            // did not recognize the query
            Ok(vec![])
        }
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
}
