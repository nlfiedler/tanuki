//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::domain::entities::Asset;
use crate::domain::repositories::RecordRepository;
use failure::Error;
use std::cmp;
use std::fmt;

pub struct FetchAsset {
    repo: Box<dyn RecordRepository>,
}

impl FetchAsset {
    pub fn new(repo: Box<dyn RecordRepository>) -> Self {
        Self { repo }
    }
}

impl super::UseCase<Asset, Params> for FetchAsset {
    fn call(&self, params: Params) -> Result<Asset, Error> {
        self.repo.get_asset(&params.asset_id)
    }
}

pub struct Params {
    asset_id: String,
}

impl Params {
    pub fn new(asset_id: String) -> Self {
        Self { asset_id }
    }
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Params({})", self.asset_id)
    }
}

impl cmp::PartialEq for Params {
    fn eq(&self, other: &Self) -> bool {
        self.asset_id == other.asset_id
    }
}

impl cmp::Eq for Params {}

#[cfg(test)]
mod tests {
    use super::super::UseCase;
    use super::*;
    use crate::domain::repositories::MockRecordRepository;
    use chrono::prelude::*;
    use failure::err_msg;
    use mockall::predicate::*;

    #[test]
    fn test_fetch_asset_ok() {
        // arrange
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
        let mut mock = MockRecordRepository::new();
        mock.expect_get_asset()
            .with(eq("abc123"))
            .returning(move |_| Ok(asset1.clone()));
        // act
        let usecase = FetchAsset::new(Box::new(mock));
        let params = Params::new("abc123".to_owned());
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let asset = result.unwrap();
        assert_eq!(asset.key, "abc123".to_owned());
    }

    #[test]
    fn test_fetch_asset_err() {
        // arrange
        let mut mock = MockRecordRepository::new();
        mock.expect_get_asset()
            .with(eq("abc123"))
            .returning(move |_| Err(err_msg("oh no")));
        // act
        let usecase = FetchAsset::new(Box::new(mock));
        let params = Params::new("abc123".to_owned());
        let result = usecase.call(params);
        // assert
        assert!(result.is_err());
    }
}
