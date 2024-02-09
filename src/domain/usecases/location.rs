//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::LabeledCount;
use crate::domain::repositories::RecordRepository;
use anyhow::Error;
use std::cmp;
use std::fmt;

pub struct AllLocations {
    repo: Box<dyn RecordRepository>,
}

impl AllLocations {
    pub fn new(repo: Box<dyn RecordRepository>) -> Self {
        Self { repo }
    }
}

impl super::UseCase<Vec<LabeledCount>, Params> for AllLocations {
    fn call(&self, params: Params) -> Result<Vec<LabeledCount>, Error> {
        if params.raw {
            self.repo.raw_locations()
        } else {
            self.repo.all_locations()
        }
    }
}

#[derive(Clone, Default)]
pub struct Params {
    /// Return the location labels as originally entered.
    pub raw: bool,
}

impl Params {
    pub fn new(raw: bool) -> Self {
        Self { raw }
    }
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Params({})", self.raw)
    }
}

impl cmp::PartialEq for Params {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl cmp::Eq for Params {}
#[cfg(test)]
mod tests {
    use super::super::UseCase;
    use super::*;
    use crate::domain::repositories::MockRecordRepository;
    use anyhow::anyhow;

    #[test]
    fn test_all_locations_ok() {
        // arrange
        let expected = vec![
            LabeledCount {
                label: "hawaii".to_owned(),
                count: 42,
            },
            LabeledCount {
                label: "paris".to_owned(),
                count: 101,
            },
            LabeledCount {
                label: "london".to_owned(),
                count: 14,
            },
        ];
        let mut mock = MockRecordRepository::new();
        mock.expect_all_locations()
            .returning(move || Ok(expected.clone()));
        // act
        let usecase = AllLocations::new(Box::new(mock));
        let params = Params { raw: false };
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(actual.len(), 3);
        assert!(actual.iter().any(|l| l.label == "hawaii" && l.count == 42));
        assert!(actual.iter().any(|l| l.label == "london" && l.count == 14));
        assert!(actual.iter().any(|l| l.label == "paris" && l.count == 101));
    }

    #[test]
    fn test_all_locations_raw() {
        // arrange
        let expected = vec![
            LabeledCount {
                label: "hawaii".to_owned(),
                count: 42,
            },
            LabeledCount {
                label: "paris, france".to_owned(),
                count: 101,
            },
            LabeledCount {
                label: "london".to_owned(),
                count: 14,
            },
        ];
        let mut mock = MockRecordRepository::new();
        mock.expect_raw_locations()
            .returning(move || Ok(expected.clone()));
        // act
        let usecase = AllLocations::new(Box::new(mock));
        let params = Params { raw: true };
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(actual.len(), 3);
        assert!(actual.iter().any(|l| l.label == "hawaii" && l.count == 42));
        assert!(actual.iter().any(|l| l.label == "london" && l.count == 14));
        assert!(actual
            .iter()
            .any(|l| l.label == "paris, france" && l.count == 101));
    }

    #[test]
    fn test_all_locations_err() {
        // arrange
        let mut mock = MockRecordRepository::new();
        mock.expect_all_locations()
            .returning(|| Err(anyhow!("oh no")));
        // act
        let usecase = AllLocations::new(Box::new(mock));
        let params = Params { raw: false };
        let result = usecase.call(params);
        // assert
        assert!(result.is_err());
    }
}
