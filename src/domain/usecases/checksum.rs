//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::domain::entities::Asset;
use crate::domain::repositories::RecordRepository;
use failure::Error;
use std::cmp;
use std::fmt;

pub struct AssetByChecksum {
    repo: Box<dyn RecordRepository>,
}

impl AssetByChecksum {
    pub fn new(repo: Box<dyn RecordRepository>) -> Self {
        Self { repo }
    }
}

impl super::UseCase<Option<Asset>, Params> for AssetByChecksum {
    fn call(&self, params: Params) -> Result<Option<Asset>, Error> {
        self.repo.get_asset_by_digest(&params.checksum)
    }
}

pub struct Params {
    checksum: String,
}

impl Params {
    pub fn new(checksum: String) -> Self {
        Self { checksum }
    }
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Params({})", self.checksum)
    }
}

impl cmp::PartialEq for Params {
    fn eq(&self, other: &Self) -> bool {
        self.checksum == other.checksum
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
            duration: None,
            user_date: None,
            original_date: None,
        };
        let mut mock = MockRecordRepository::new();
        mock.expect_get_asset_by_digest()
            .with(eq("cafebabe"))
            .returning(move |_| Ok(Some(asset1.clone())));
        // act
        let usecase = AssetByChecksum::new(Box::new(mock));
        let params = Params::new("cafebabe".to_owned());
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let option = result.unwrap();
        assert!(option.is_some());
        assert_eq!(option.unwrap().key, "abc123");
    }

    #[test]
    fn test_fetch_asset_err() {
        // arrange
        let mut mock = MockRecordRepository::new();
        mock.expect_get_asset_by_digest()
            .with(eq("cafebabe"))
            .returning(move |_| Err(err_msg("oh no")));
        // act
        let usecase = AssetByChecksum::new(Box::new(mock));
        let params = Params::new("cafebabe".to_owned());
        let result = usecase.call(params);
        // assert
        assert!(result.is_err());
    }
}
