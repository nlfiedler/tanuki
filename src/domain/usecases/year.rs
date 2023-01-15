//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::domain::entities::LabeledCount;
use crate::domain::repositories::RecordRepository;
use crate::domain::usecases::NoParams;
use anyhow::Error;

pub struct AllYears {
    repo: Box<dyn RecordRepository>,
}

impl AllYears {
    pub fn new(repo: Box<dyn RecordRepository>) -> Self {
        Self { repo }
    }
}

impl super::UseCase<Vec<LabeledCount>, NoParams> for AllYears {
    fn call(&self, _params: NoParams) -> Result<Vec<LabeledCount>, Error> {
        self.repo.all_years()
    }
}

#[cfg(test)]
mod tests {
    use super::super::{NoParams, UseCase};
    use super::*;
    use crate::domain::repositories::MockRecordRepository;
    use anyhow::anyhow;

    #[test]
    fn test_all_years_ok() {
        // arrange
        let expected = vec![
            LabeledCount {
                label: "1996".to_owned(),
                count: 42,
            },
            LabeledCount {
                label: "2006".to_owned(),
                count: 101,
            },
            LabeledCount {
                label: "2016".to_owned(),
                count: 14,
            },
        ];
        let mut mock = MockRecordRepository::new();
        mock.expect_all_years()
            .returning(move || Ok(expected.clone()));
        // act
        let usecase = AllYears::new(Box::new(mock));
        let params = NoParams {};
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(actual.len(), 3);
        assert!(actual.iter().any(|l| l.label == "1996" && l.count == 42));
        assert!(actual.iter().any(|l| l.label == "2006" && l.count == 101));
        assert!(actual.iter().any(|l| l.label == "2016" && l.count == 14));
    }

    #[test]
    fn test_all_years_err() {
        // arrange
        let mut mock = MockRecordRepository::new();
        mock.expect_all_years().returning(|| Err(anyhow!("oh no")));
        // act
        let usecase = AllYears::new(Box::new(mock));
        let params = NoParams {};
        let result = usecase.call(params);
        // assert
        assert!(result.is_err());
    }
}
