//
// Copyright (c) 2023 Nathan Fiedler
//
use crate::domain::entities::{SearchResult, SortField, SortOrder};
use crate::domain::repositories::RecordRepository;
use anyhow::Error;
use chrono::prelude::*;
use hashed_array_tree::HashedArrayTree;
use std::cmp;
use std::fmt;

/// Use case to recently imported assets (i.e. "newborn").
pub struct RecentImports {
    repo: Box<dyn RecordRepository>,
}

impl RecentImports {
    pub fn new(repo: Box<dyn RecordRepository>) -> Self {
        Self { repo }
    }
}

impl super::UseCase<HashedArrayTree<SearchResult>, Params> for RecentImports {
    fn call(&self, params: Params) -> Result<HashedArrayTree<SearchResult>, Error> {
        let after = params.after_date.unwrap_or_else(|| {
            let date = chrono::DateTime::<Utc>::MIN_UTC;
            Utc.with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
                .earliest()
                .unwrap_or_else(Utc::now)
        });
        let mut results = self.repo.query_newborn(after)?;
        super::sort_results(&mut results, params.sort_field, params.sort_order);
        Ok(results)
    }
}

#[derive(Clone, Default)]
pub struct Params {
    pub after_date: Option<DateTime<Utc>>,
    pub sort_field: Option<SortField>,
    pub sort_order: Option<SortOrder>,
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Params(after: {:?})", self.after_date)
    }
}

impl cmp::PartialEq for Params {
    fn eq(&self, other: &Self) -> bool {
        self.after_date == other.after_date
    }
}

impl cmp::Eq for Params {}

#[cfg(test)]
mod tests {
    use super::super::UseCase;
    use super::*;
    use crate::domain::repositories::MockRecordRepository;
    use anyhow::anyhow;
    use hashed_array_tree::hat;
    use mockall::predicate::*;

    #[test]
    fn test_recent_imports_err() {
        // arrange
        let mut mock = MockRecordRepository::new();
        mock.expect_query_newborn()
            .returning(move |_| Err(anyhow!("oh no")));
        // act
        let usecase = RecentImports::new(Box::new(mock));
        let params: Params = Default::default();
        let result = usecase.call(params);
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_recent_imports_alltime_ok() {
        // arrange
        let results = hat![SearchResult {
            asset_id: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: None,
            datetime: Utc::now(),
        }];
        let mut mock = MockRecordRepository::new();
        mock.expect_query_newborn()
            .with(always())
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = RecentImports::new(Box::new(mock));
        let params: Params = Default::default();
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    #[test]
    fn test_recent_imports_after_ok() {
        // arrange
        let results = hat![SearchResult {
            asset_id: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: None,
            datetime: Utc::now(),
        }];
        let after = Utc
            .with_ymd_and_hms(2018, 5, 31, 21, 10, 11)
            .single()
            .unwrap();
        let mut mock = MockRecordRepository::new();
        mock.expect_query_newborn()
            .with(eq(after))
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = RecentImports::new(Box::new(mock));
        let params: Params = Params {
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
}
