//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::data::repositories::RecordRepositoryImpl;
use crate::data::sources::EntityDataSource;
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
}
