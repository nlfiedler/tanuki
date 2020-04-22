//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::domain::entities::LabeledCount;
use crate::domain::repositories::RecordRepository;
use crate::domain::usecases::NoParams;
use failure::Error;

pub struct AllTags {
    repo: Box<dyn RecordRepository>,
}

impl AllTags {
    pub fn new(repo: Box<dyn RecordRepository>) -> Self {
        Self { repo }
    }
}

impl super::UseCase<Vec<LabeledCount>, NoParams> for AllTags {
    fn call(&self, _params: NoParams) -> Result<Vec<LabeledCount>, Error> {
        self.repo.all_tags()
    }
}

#[cfg(test)]
mod tests {
    use super::super::{NoParams, UseCase};
    use super::*;
    use crate::domain::repositories::MockRecordRepository;
    use failure::err_msg;

    #[test]
    fn test_all_tags_ok() {
        // arrange
        let expected = vec![
            LabeledCount {
                label: "dog".to_owned(),
                count: 42,
            },
            LabeledCount {
                label: "cat".to_owned(),
                count: 101,
            },
            LabeledCount {
                label: "mouse".to_owned(),
                count: 14,
            },
        ];
        let mut mock = MockRecordRepository::new();
        mock.expect_all_tags()
            .with()
            .returning(move || Ok(expected.clone()));
        // act
        let usecase = AllTags::new(Box::new(mock));
        let params = NoParams {};
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(actual.len(), 3);
        assert!(actual.iter().any(|l| l.label == "dog" && l.count == 42));
        assert!(actual.iter().any(|l| l.label == "cat" && l.count == 101));
        assert!(actual.iter().any(|l| l.label == "mouse" && l.count == 14));
    }

    #[test]
    fn test_all_tags_err() {
        // arrange
        let mut mock = MockRecordRepository::new();
        mock.expect_all_tags()
            .with()
            .returning(|| Err(err_msg("oh no")));
        // act
        let usecase = AllTags::new(Box::new(mock));
        let params = NoParams {};
        let result = usecase.call(params);
        // assert
        assert!(result.is_err());
    }
}
