//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::domain::entities::LabeledCount;
use crate::domain::repositories::RecordRepository;
use crate::domain::usecases::NoParams;
use failure::Error;

pub struct AllLocations {
    repo: Box<dyn RecordRepository>,
}

impl AllLocations {
    pub fn new(repo: Box<dyn RecordRepository>) -> Self {
        Self { repo }
    }
}

impl super::UseCase<Vec<LabeledCount>, NoParams> for AllLocations {
    fn call(&self, _params: NoParams) -> Result<Vec<LabeledCount>, Error> {
        self.repo.all_locations()
    }
}

#[cfg(test)]
mod tests {
    use super::super::{NoParams, UseCase};
    use super::*;
    use crate::domain::repositories::MockRecordRepository;
    use failure::err_msg;

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
            .with()
            .returning(move || Ok(expected.clone()));
        // act
        let usecase = AllLocations::new(Box::new(mock));
        let params = NoParams {};
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
    fn test_all_locations_err() {
        // arrange
        let mut mock = MockRecordRepository::new();
        mock.expect_all_locations()
            .with()
            .returning(|| Err(err_msg("oh no")));
        // act
        let usecase = AllLocations::new(Box::new(mock));
        let params = NoParams {};
        let result = usecase.call(params);
        // assert
        assert!(result.is_err());
    }
}
