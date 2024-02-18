//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::{LabeledCount, Location};
use crate::domain::repositories::RecordRepository;
use crate::domain::usecases::NoParams;
use anyhow::Error;

///
/// Returns the various location values and their respective counts. This
/// includes location labels that have been split on commas, the location city
/// and region, all returned as separate elements.
///
pub struct PartedLocations {
    repo: Box<dyn RecordRepository>,
}

impl PartedLocations {
    pub fn new(repo: Box<dyn RecordRepository>) -> Self {
        Self { repo }
    }
}

impl super::UseCase<Vec<LabeledCount>, NoParams> for PartedLocations {
    fn call(&self, _params: NoParams) -> Result<Vec<LabeledCount>, Error> {
        self.repo.all_locations()
    }
}

///
/// Returns the unique location records with their full structure and original
/// values.
///
pub struct CompleteLocations {
    repo: Box<dyn RecordRepository>,
}

impl CompleteLocations {
    pub fn new(repo: Box<dyn RecordRepository>) -> Self {
        Self { repo }
    }
}

impl super::UseCase<Vec<Location>, NoParams> for CompleteLocations {
    fn call(&self, _params: NoParams) -> Result<Vec<Location>, Error> {
        self.repo.raw_locations()
    }
}

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
        let usecase = PartedLocations::new(Box::new(mock));
        let result = usecase.call(NoParams {});
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
        let input = vec![
            Location {
                label: Some("beach".to_owned()),
                city: Some("Waikiki".into()),
                region: Some("Hawaii".into()),
            },
            Location {
                label: None,
                city: Some("Portland".into()),
                region: Some("Oregon".into()),
            },
            Location {
                label: Some("black sands beach".to_owned()),
                city: None,
                region: None,
            },
        ];
        let expected = input.clone();
        let mut mock = MockRecordRepository::new();
        mock.expect_raw_locations()
            .returning(move || Ok(input.clone()));
        // act
        let usecase = CompleteLocations::new(Box::new(mock));
        let result = usecase.call(NoParams {});
        // assert
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(actual.len(), 3);
        assert_eq!(actual[0], expected[0]);
        assert_eq!(actual[1], expected[1]);
        assert_eq!(actual[2], expected[2]);
    }

    #[test]
    fn test_all_locations_err() {
        // arrange
        let mut mock = MockRecordRepository::new();
        mock.expect_all_locations()
            .returning(|| Err(anyhow!("oh no")));
        // act
        let usecase = PartedLocations::new(Box::new(mock));
        let result = usecase.call(NoParams {});
        // assert
        assert!(result.is_err());
    }
}
