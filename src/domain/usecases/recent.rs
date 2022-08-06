//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::domain::entities::SearchResult;
use crate::domain::repositories::RecordRepository;
use chrono::prelude::*;
use anyhow::Error;
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

impl super::UseCase<Vec<SearchResult>, Params> for RecentImports {
    fn call(&self, params: Params) -> Result<Vec<SearchResult>, Error> {
        let after = params.after_date.unwrap_or_else(|| {
            let date = chrono::Date::<Utc>::MIN_UTC;
            Utc.ymd(date.year(), date.month(), date.day())
                .and_hms(0, 0, 0)
        });
        let results = self.repo.query_newborn(after)?;
        Ok(results)
    }
}

#[derive(Clone, Default)]
pub struct Params {
    pub after_date: Option<DateTime<Utc>>,
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
        let results = vec![SearchResult {
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
        let results = vec![SearchResult {
            asset_id: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: None,
            datetime: Utc::now(),
        }];
        let after = Utc.ymd(2018, 5, 31).and_hms(21, 10, 11);
        let mut mock = MockRecordRepository::new();
        mock.expect_query_newborn()
            .with(eq(after))
            .returning(move |_| Ok(results.clone()));
        // act
        let usecase = RecentImports::new(Box::new(mock));
        let mut params: Params = Default::default();
        params.after_date = Some(after);
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }
}
