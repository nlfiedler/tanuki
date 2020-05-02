//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::data::repositories::RecordRepositoryImpl;
use crate::data::sources::EntityDataSource;
use crate::domain::entities::LabeledCount;
use juniper::{graphql_scalar, FieldResult, ParseScalarResult, ParseScalarValue, RootNode, Value};
use std::sync::Arc;

// Mark the data source as a valid context type for Juniper.
impl juniper::Context for dyn EntityDataSource {}

// Define a larger integer type so we can represent those larger values, such as
// file sizes. Some of the core types define fields that are larger than i32, so
// this type is used to represent those values in GraphQL.
#[derive(Copy, Clone)]
pub struct BigInt(i64);

impl BigInt {
    /// Construct a BigInt for the given value.
    pub fn new(value: i64) -> Self {
        BigInt(value)
    }
}

impl Into<u32> for BigInt {
    fn into(self) -> u32 {
        self.0 as u32
    }
}

impl Into<u64> for BigInt {
    fn into(self) -> u64 {
        self.0 as u64
    }
}

impl From<u32> for BigInt {
    fn from(t: u32) -> Self {
        BigInt(i64::from(t))
    }
}

// need `where Scalar = <S>` parameterization to use this with objects
// c.f. https://github.com/graphql-rust/juniper/issues/358 for details
graphql_scalar!(BigInt where Scalar = <S> {
    description: "An integer type larger than the standard signed 32-bit."

    resolve(&self) -> Value {
        Value::scalar(format!("{}", self.0))
    }

    from_input_value(v: &InputValue) -> Option<BigInt> {
        v.as_scalar_value::<String>().filter(|s| {
            // make sure the input value parses as an integer
            i64::from_str_radix(s, 10).is_ok()
        }).map(|s| BigInt(i64::from_str_radix(s, 10).unwrap()))
    }

    from_str<'a>(value: ScalarToken<'a>) -> ParseScalarResult<'a, S> {
        <String as ParseScalarValue<S>>::from_str(value)
    }
});

#[juniper::object(description = "An attribute name and the number of assets it references.")]
impl LabeledCount {
    /// Label for an asset attribute, such as a tag or location.
    fn label(&self) -> String {
        self.label.clone()
    }

    /// Number of assets that are associated with this particular label.
    fn count(&self) -> i32 {
        self.count as i32
    }
}

pub struct QueryRoot;

#[juniper::object(Context = Arc<dyn EntityDataSource>)]
impl QueryRoot {
    /// Return the total number of assets in the system.
    fn count(executor: &Executor) -> FieldResult<i32> {
        use crate::domain::usecases::count::CountAssets;
        use crate::domain::usecases::{NoParams, UseCase};
        let source = executor.context().clone();
        let repo = RecordRepositoryImpl::new(source);
        let usecase = CountAssets::new(Box::new(repo));
        let params = NoParams {};
        let count = usecase.call(params)?;
        Ok(count as i32)
    }

    /// Retrieve the list of locations and their associated asset count.
    fn locations(executor: &Executor) -> FieldResult<Vec<LabeledCount>> {
        use crate::domain::usecases::location::AllLocations;
        use crate::domain::usecases::{NoParams, UseCase};
        let source = executor.context().clone();
        let repo = RecordRepositoryImpl::new(source);
        let usecase = AllLocations::new(Box::new(repo));
        let params = NoParams {};
        let locations: Vec<LabeledCount> = usecase.call(params)?;
        Ok(locations)
    }

    /// Retrieve the list of tags and their associated asset count.
    fn tags(executor: &Executor) -> FieldResult<Vec<LabeledCount>> {
        use crate::domain::usecases::tags::AllTags;
        use crate::domain::usecases::{NoParams, UseCase};
        let source = executor.context().clone();
        let repo = RecordRepositoryImpl::new(source);
        let usecase = AllTags::new(Box::new(repo));
        let params = NoParams {};
        let tags: Vec<LabeledCount> = usecase.call(params)?;
        Ok(tags)
    }

    /// Retrieve the list of years and their associated asset count.
    fn years(executor: &Executor) -> FieldResult<Vec<LabeledCount>> {
        use crate::domain::usecases::year::AllYears;
        use crate::domain::usecases::{NoParams, UseCase};
        let source = executor.context().clone();
        let repo = RecordRepositoryImpl::new(source);
        let usecase = AllYears::new(Box::new(repo));
        let params = NoParams {};
        let years: Vec<LabeledCount> = usecase.call(params)?;
        Ok(years)
    }
}

pub struct MutationRoot;

#[juniper::object(Context = Arc<dyn EntityDataSource>)]
impl MutationRoot {}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

/// Create the GraphQL schema.
pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::sources::MockEntityDataSource;
    use failure::err_msg;
    use juniper::Variables;
    // use mockall::predicate::*;

    #[test]
    fn test_query_count_ok() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_count_assets().with().returning(|| Ok(42));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let (res, _errors) =
            juniper::execute(r#"query { count }"#, None, &schema, &Variables::new(), &ctx).unwrap();
        // assert
        let res = res.as_object_value().unwrap();
        let res = res.get_field_value("count").unwrap();
        let actual = res.as_scalar_value::<i32>().unwrap();
        assert_eq!(*actual, 42);
    }

    #[test]
    fn test_query_count_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_count_assets()
            .with()
            .returning(|| Err(err_msg("oh no")));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let (res, errors) =
            juniper::execute(r#"query { count }"#, None, &schema, &Variables::new(), &ctx).unwrap();
        // assert
        assert!(res.is_null());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].error().message().contains("oh no"));
    }

    #[test]
    fn test_query_locations_ok() {
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
        let mut mock = MockEntityDataSource::new();
        mock.expect_all_locations()
            .with()
            .returning(move || Ok(expected.clone()));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let (res, _errors) = juniper::execute(
            r#"query { locations { label count } }"#,
            None,
            &schema,
            &Variables::new(),
            &ctx,
        )
        .unwrap();
        // assert
        let res = res.as_object_value().unwrap();
        let res = res.get_field_value("locations").unwrap();
        let list_result = res.as_list_value().unwrap();
        let labels = ["hawaii", "paris", "london"];
        let counts = [42, 101, 14];
        for (idx, result) in list_result.iter().enumerate() {
            let object = result.as_object_value().unwrap();
            let res = object.get_field_value("label").unwrap();
            let actual = res.as_scalar_value::<String>().unwrap();
            assert_eq!(actual, labels[idx]);
            let res = object.get_field_value("count").unwrap();
            let actual = res.as_scalar_value::<i32>().unwrap();
            assert_eq!(*actual, counts[idx]);
        }
    }

    #[test]
    fn test_query_locations_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_all_locations()
            .with()
            .returning(|| Err(err_msg("oh no")));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let (res, errors) = juniper::execute(
            r#"query { locations { label count } }"#,
            None,
            &schema,
            &Variables::new(),
            &ctx,
        )
        .unwrap();
        // assert
        assert!(res.is_null());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].error().message().contains("oh no"));
    }

    #[test]
    fn test_query_tags_ok() {
        // arrange
        let expected = vec![
            LabeledCount {
                label: "cat".to_owned(),
                count: 42,
            },
            LabeledCount {
                label: "dog".to_owned(),
                count: 101,
            },
            LabeledCount {
                label: "mouse".to_owned(),
                count: 14,
            },
        ];
        let mut mock = MockEntityDataSource::new();
        mock.expect_all_tags()
            .with()
            .returning(move || Ok(expected.clone()));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let (res, _errors) = juniper::execute(
            r#"query { tags { label count } }"#,
            None,
            &schema,
            &Variables::new(),
            &ctx,
        )
        .unwrap();
        // assert
        let res = res.as_object_value().unwrap();
        let res = res.get_field_value("tags").unwrap();
        let list_result = res.as_list_value().unwrap();
        let labels = ["cat", "dog", "mouse"];
        let counts = [42, 101, 14];
        for (idx, result) in list_result.iter().enumerate() {
            let object = result.as_object_value().unwrap();
            let res = object.get_field_value("label").unwrap();
            let actual = res.as_scalar_value::<String>().unwrap();
            assert_eq!(actual, labels[idx]);
            let res = object.get_field_value("count").unwrap();
            let actual = res.as_scalar_value::<i32>().unwrap();
            assert_eq!(*actual, counts[idx]);
        }
    }

    #[test]
    fn test_query_tags_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_all_tags()
            .with()
            .returning(|| Err(err_msg("oh no")));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let (res, errors) = juniper::execute(
            r#"query { tags { label count } }"#,
            None,
            &schema,
            &Variables::new(),
            &ctx,
        )
        .unwrap();
        // assert
        assert!(res.is_null());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].error().message().contains("oh no"));
    }

    #[test]
    fn test_query_years_ok() {
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
        let mut mock = MockEntityDataSource::new();
        mock.expect_all_years()
            .with()
            .returning(move || Ok(expected.clone()));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let (res, _errors) = juniper::execute(
            r#"query { years { label count } }"#,
            None,
            &schema,
            &Variables::new(),
            &ctx,
        )
        .unwrap();
        // assert
        let res = res.as_object_value().unwrap();
        let res = res.get_field_value("years").unwrap();
        let list_result = res.as_list_value().unwrap();
        let labels = ["1996", "2006", "2016"];
        let counts = [42, 101, 14];
        for (idx, result) in list_result.iter().enumerate() {
            let object = result.as_object_value().unwrap();
            let res = object.get_field_value("label").unwrap();
            let actual = res.as_scalar_value::<String>().unwrap();
            assert_eq!(actual, labels[idx]);
            let res = object.get_field_value("count").unwrap();
            let actual = res.as_scalar_value::<i32>().unwrap();
            assert_eq!(*actual, counts[idx]);
        }
    }

    #[test]
    fn test_query_years_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_all_years()
            .with()
            .returning(|| Err(err_msg("oh no")));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let (res, errors) = juniper::execute(
            r#"query { years { label count } }"#,
            None,
            &schema,
            &Variables::new(),
            &ctx,
        )
        .unwrap();
        // assert
        assert!(res.is_null());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].error().message().contains("oh no"));
    }
}
