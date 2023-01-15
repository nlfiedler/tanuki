//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::domain::repositories::RecordRepository;
use crate::domain::usecases::NoParams;
use anyhow::Error;

pub struct CountAssets {
    repo: Box<dyn RecordRepository>,
}

impl CountAssets {
    pub fn new(repo: Box<dyn RecordRepository>) -> Self {
        Self { repo }
    }
}

impl super::UseCase<u64, NoParams> for CountAssets {
    fn call(&self, _params: NoParams) -> Result<u64, Error> {
        self.repo.count_assets()
    }
}

#[cfg(test)]
mod tests {
    use super::super::{NoParams, UseCase};
    use super::*;
    use crate::domain::repositories::MockRecordRepository;
    use anyhow::anyhow;

    #[test]
    fn test_count_assets_ok() {
        // arrange
        let mut mock = MockRecordRepository::new();
        mock.expect_count_assets().returning(|| Ok(42));
        // act
        let usecase = CountAssets::new(Box::new(mock));
        let params = NoParams {};
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_count_assets_err() {
        // arrange
        let mut mock = MockRecordRepository::new();
        mock.expect_count_assets()
            .returning(|| Err(anyhow!("oh no")));
        // act
        let usecase = CountAssets::new(Box::new(mock));
        let params = NoParams {};
        let result = usecase.call(params);
        // assert
        assert!(result.is_err());
    }
}
