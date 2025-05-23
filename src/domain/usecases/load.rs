//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::repositories::{RecordRepository, SearchRepository};
use anyhow::Error;
use log::info;
use std::cmp;
use std::fmt;
use std::path::PathBuf;

pub struct Load {
    records: Box<dyn RecordRepository>,
    cache: Box<dyn SearchRepository>,
}

impl Load {
    pub fn new(records: Box<dyn RecordRepository>, cache: Box<dyn SearchRepository>) -> Self {
        Self { records, cache }
    }
}

impl super::UseCase<u64, Params> for Load {
    fn call(&self, params: Params) -> Result<u64, Error> {
        info!("load commencing...");
        let count = self.records.load(&params.filepath)?;
        self.cache.clear()?;
        info!("load complete");
        Ok(count)
    }
}

#[derive(Clone)]
pub struct Params {
    /// Path where the JSON formatted output should be written.
    filepath: PathBuf,
}

impl Params {
    pub fn new(filepath: PathBuf) -> Self {
        Self { filepath }
    }
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Params({:?})", self.filepath)
    }
}

impl cmp::PartialEq for Params {
    fn eq(&self, other: &Self) -> bool {
        self.filepath == other.filepath
    }
}

impl cmp::Eq for Params {}

#[cfg(test)]
mod tests {
    use super::super::UseCase;
    use super::*;
    use crate::domain::repositories::{MockRecordRepository, MockSearchRepository};
    use anyhow::anyhow;

    #[test]
    fn test_load_ok() {
        // arrange
        let mut records = MockRecordRepository::new();
        records.expect_load().returning(|_| Ok(101));
        let mut cache = MockSearchRepository::new();
        cache.expect_clear().returning(|| Ok(()));
        // act
        let usecase = Load::new(Box::new(records), Box::new(cache));
        let params = Params::new("dump.json".into());
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 101);
    }

    #[test]
    fn test_load_err() {
        // arrange
        let mut records = MockRecordRepository::new();
        records.expect_load().returning(|_| Err(anyhow!("oh no")));
        let mut cache = MockSearchRepository::new();
        cache.expect_clear().returning(|| Ok(()));
        // act
        let usecase = Load::new(Box::new(records), Box::new(cache));
        let params = Params::new("dump.json".into());
        let result = usecase.call(params);
        // assert
        assert!(result.is_err());
    }
}
