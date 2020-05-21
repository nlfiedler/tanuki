//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::data::repositories::RecordRepositoryImpl;
use crate::data::sources::EntityDataSource;
use crate::domain::entities::{Asset, LabeledCount, SearchResult};
use chrono::prelude::*;
use juniper::{
    graphql_scalar, FieldResult, GraphQLEnum, GraphQLInputObject, ParseScalarResult,
    ParseScalarValue, RootNode, Value,
};
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

#[juniper::object(description = "An `Asset` defines a single entity in the storage system.")]
impl Asset {
    /// The unique asset identifier.
    fn id(&self) -> String {
        self.key.clone()
    }

    /// The original filename of the asset when it was imported.
    fn filename(&self) -> String {
        self.filename.clone()
    }

    /// The size in bytes of the asset.
    fn filesize(&self) -> BigInt {
        BigInt(self.byte_length as i64)
    }

    /// The date/time that best represents the asset.
    fn datetime(&self) -> DateTime<Utc> {
        if let Some(ud) = self.user_date.as_ref() {
            ud.clone()
        } else if let Some(od) = self.original_date.as_ref() {
            od.clone()
        } else {
            self.import_date.clone()
        }
    }

    /// The media type (nee MIME type) of the asset.
    fn mimetype(&self) -> String {
        self.media_type.clone()
    }

    /// The list of tags associated with this asset.
    fn tags(&self) -> Vec<String> {
        self.tags.clone()
    }

    /// The date provided by the user.
    fn userdate(&self) -> Option<DateTime<Utc>> {
        self.user_date.clone()
    }

    /// A caption attributed to the asset.
    fn caption(&self) -> Option<String> {
        self.caption.clone()
    }

    /// Location information for the asset.
    fn location(&self) -> Option<String> {
        self.location.clone()
    }
}

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

#[derive(GraphQLEnum)]
pub enum SortField {
    Date,
    Identifier,
    Filename,
    MediaType,
    Location,
}

impl Into<crate::domain::usecases::search::SortField> for SortField {
    fn into(self) -> crate::domain::usecases::search::SortField {
        match self {
            SortField::Date => crate::domain::usecases::search::SortField::Date,
            SortField::Identifier => crate::domain::usecases::search::SortField::Identifier,
            SortField::Filename => crate::domain::usecases::search::SortField::Filename,
            SortField::MediaType => crate::domain::usecases::search::SortField::MediaType,
            SortField::Location => crate::domain::usecases::search::SortField::Location,
        }
    }
}

#[derive(GraphQLEnum)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl Into<crate::domain::usecases::search::SortOrder> for SortOrder {
    fn into(self) -> crate::domain::usecases::search::SortOrder {
        match self {
            SortOrder::Ascending => crate::domain::usecases::search::SortOrder::Ascending,
            SortOrder::Descending => crate::domain::usecases::search::SortOrder::Descending,
        }
    }
}

/// `SearchParams` defines the various parameters by which to search for assets.
#[derive(GraphQLInputObject)]
pub struct SearchParams {
    /// Tags that an asset should have. All should match.
    pub tags: Option<Vec<String>>,
    /// Locations of an asset. At least one must match.
    pub locations: Option<Vec<String>>,
    /// Date for filtering asset results. Only those assets whose canonical date
    /// occurs _on_ or _after_ this date will be returned.
    pub after: Option<DateTime<Utc>>,
    /// Date for filtering asset results. Only those assets whose canonical date
    /// occurs _before_ this date will be returned.
    pub before: Option<DateTime<Utc>>,
    /// Find assets whose filename (e.g. `img_3011.jpg`) matches the one given.
    pub filename: Option<String>,
    /// Find assets whose mimetype (e.g. `image/jpeg`) matches the one given.
    pub mimetype: Option<String>,
    /// Field by which to sort the results.
    pub sort_field: Option<SortField>,
    /// Order by which to sort the results.
    pub sort_order: Option<SortOrder>,
}

impl Into<crate::domain::usecases::search::Params> for SearchParams {
    fn into(self) -> crate::domain::usecases::search::Params {
        crate::domain::usecases::search::Params {
            tags: self.tags.unwrap_or(vec![]),
            locations: self.locations.unwrap_or(vec![]),
            filename: self.filename,
            mimetype: self.mimetype,
            before_date: self.before,
            after_date: self.after,
            sort_field: Some(
                self.sort_field
                    .map_or(crate::domain::usecases::search::SortField::Date, |v| {
                        v.into()
                    }),
            ),
            sort_order: Some(self.sort_order.map_or(
                crate::domain::usecases::search::SortOrder::Descending,
                |v| v.into(),
            )),
        }
    }
}

#[juniper::object(description = "An attribute name and the number of assets it references.")]
impl SearchResult {
    /// The identifier of the matching asset.
    fn id(&self) -> String {
        self.asset_id.clone()
    }

    /// The filename for the matching asset.
    fn filename(&self) -> String {
        self.filename.clone()
    }

    /// Media type (formerly MIME type) of the asset.
    fn mimetype(&self) -> String {
        self.media_type.clone()
    }

    /// The location for the matching asset, if available.
    fn location(&self) -> Option<String> {
        self.location.clone()
    }

    /// The date/time for the matching asset.
    fn datetime(&self) -> DateTime<Utc> {
        self.datetime.clone()
    }
}

struct SearchMeta {
    results: Vec<SearchResult>,
    count: i32,
}

#[juniper::object(description = "`SearchMeta` is returned from the `search` query.")]
impl SearchMeta {
    /// The list of results retrieved via the query.
    fn results(&self) -> Vec<SearchResult> {
        self.results.clone()
    }

    /// The total number of matching assets in the system, useful for pagination.
    fn count(&self) -> i32 {
        self.count
    }
}

/// `AssetInput` is used to update the details of an asset.
#[derive(Clone, GraphQLInputObject)]
pub struct AssetInput {
    /// New set of tags to replace the existing set.
    tags: Option<Vec<String>>,
    /// New caption to replace any existing value. Note that the caption text
    /// may contain hashtags which are merged with the other tags, and may have
    /// an @location that sets the location, if it has not yet defined.
    caption: Option<String>,
    /// New location to replace any existing value.
    location: Option<String>,
    /// A date/time that overrides intrinsic values; a `null` clears the custom
    /// field and reverts back to the intrinsic value.
    datetime: Option<DateTime<Utc>>,
    /// New media type, useful for fixing assets where the automatic detection
    /// guessed wrong. Beware that setting a wrong value means the asset will
    /// likely not display correctly.
    mimetype: Option<String>,
}

impl Into<crate::domain::usecases::update::AssetInput> for AssetInput {
    fn into(self) -> crate::domain::usecases::update::AssetInput {
        let mut tags = self.tags.unwrap_or(vec![]);
        // Filter out empty tags, as the front-end may send those because it is
        // too lazy to filter them itself.
        tags = tags.iter().filter(|t| t.len() > 0).cloned().collect();
        crate::domain::usecases::update::AssetInput {
            tags,
            caption: self.caption,
            location: self.location,
            media_type: self.mimetype,
            datetime: self.datetime,
        }
    }
}

/// `AssetInputId` is used to update the details of an asset.
#[derive(GraphQLInputObject)]
pub struct AssetInputId {
    /// Identifier for the asset to be updated.
    id: String,
    /// Input for the asset.
    input: AssetInput,
}

pub struct QueryRoot;

#[juniper::object(Context = Arc<dyn EntityDataSource>)]
impl QueryRoot {
    /// Retrieve an asset by its unique identifier.
    fn asset(executor: &Executor, id: String) -> FieldResult<Asset> {
        use crate::domain::usecases::fetch::{FetchAsset, Params};
        use crate::domain::usecases::UseCase;
        let source = executor.context().clone();
        let repo = RecordRepositoryImpl::new(source);
        let usecase = FetchAsset::new(Box::new(repo));
        let params = Params::new(id);
        let asset = usecase.call(params)?;
        Ok(asset)
    }

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

    /// Look for an asset by the hash digest (SHA256).
    fn lookup(executor: &Executor, checksum: String) -> FieldResult<Option<Asset>> {
        use crate::domain::usecases::checksum::{AssetByChecksum, Params};
        use crate::domain::usecases::UseCase;
        let source = executor.context().clone();
        let repo = RecordRepositoryImpl::new(source);
        let usecase = AssetByChecksum::new(Box::new(repo));
        let params = Params::new(checksum);
        let asset = usecase.call(params)?;
        Ok(asset)
    }

    /// Search for assets that were recently imported.
    ///
    /// Recently imported assets do not have any tags, location, or caption, and
    /// thus are waiting for the user to give them additional details.
    fn recent(executor: &Executor, since: Option<DateTime<Utc>>) -> FieldResult<SearchMeta> {
        use crate::domain::usecases::recent::{Params, RecentImports};
        use crate::domain::usecases::UseCase;
        let source = executor.context().clone();
        let repo = RecordRepositoryImpl::new(source);
        let usecase = RecentImports::new(Box::new(repo));
        let mut params: Params = Default::default();
        params.after_date = since;
        let results: Vec<SearchResult> = usecase.call(params)?;
        let total_count = results.len() as i32;
        Ok(SearchMeta {
            results,
            count: total_count,
        })
    }

    /// Search for assets by the given parameters.
    ///
    /// The count indicates how many results to return in a single query,
    /// limited to a maximum of 250. Default value is `10`.
    ///
    /// The offset is useful for pagination. Default value is `0`.
    fn search(
        executor: &Executor,
        params: SearchParams,
        count: Option<i32>,
        offset: Option<i32>,
    ) -> FieldResult<SearchMeta> {
        use crate::domain::usecases::search::{Params, SearchAssets};
        use crate::domain::usecases::UseCase;
        let source = executor.context().clone();
        let repo = RecordRepositoryImpl::new(source);
        let usecase = SearchAssets::new(Box::new(repo));
        let params: Params = params.into();
        let mut results: Vec<SearchResult> = usecase.call(params)?;
        let total_count = results.len() as i32;
        let results = paginate_vector(&mut results, offset, count);
        Ok(SearchMeta {
            results,
            count: total_count,
        })
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

/// Return the optional value bounded by the given range, or the default value
/// if `value` is `None`.
fn bounded_int_value(value: Option<i32>, default: i32, minimum: i32, maximum: i32) -> i32 {
    if let Some(v) = value {
        std::cmp::min(std::cmp::max(v, minimum), maximum)
    } else {
        default
    }
}

/// Truncate the given vector to yield the desired portion.
///
/// If offset is None, it defaults to 0, while count defaults to 10. Offset is
/// bound between zero and the length of the input vector. Count is bound by 1
/// and 250.
fn paginate_vector<T>(input: &mut Vec<T>, offset: Option<i32>, count: Option<i32>) -> Vec<T> {
    let total_count = input.len() as i32;
    let count = bounded_int_value(count, 10, 1, 250) as usize;
    let offset = bounded_int_value(offset, 0, 0, total_count) as usize;
    let mut results = input.split_off(offset);
    results.truncate(count);
    results
}

pub struct MutationRoot;

#[juniper::object(Context = Arc<dyn EntityDataSource>)]
impl MutationRoot {
    /// Update the asset with the given values.
    fn update(executor: &Executor, id: String, asset: AssetInput) -> FieldResult<Asset> {
        use crate::domain::usecases::update::{Params, UpdateAsset};
        use crate::domain::usecases::UseCase;
        let source = executor.context().clone();
        let repo = RecordRepositoryImpl::new(source);
        let usecase = UpdateAsset::new(Box::new(repo));
        let params: Params = Params::new(id, asset.into());
        let result: Asset = usecase.call(params)?;
        Ok(result)
    }

    /// Update multiple assets with the given values.
    ///
    /// Returns the number of updated assets.
    fn bulk_update(executor: &Executor, assets: Vec<AssetInputId>) -> FieldResult<i32> {
        use crate::domain::usecases::update::{Params, UpdateAsset};
        use crate::domain::usecases::UseCase;
        let source = executor.context().clone();
        let repo = RecordRepositoryImpl::new(source);
        let usecase = UpdateAsset::new(Box::new(repo));
        for asset in assets.iter() {
            let params: Params = Params::new(asset.id.clone(), asset.input.clone().into());
            let result: Asset = usecase.call(params)?;
        }
        Ok(assets.len() as i32)
    }
}

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
    use juniper::{InputValue, ToInputValue, Variables};
    use mockall::predicate::*;

    #[test]
    fn test_bounded_int_value() {
        assert_eq!(10, bounded_int_value(None, 10, 1, 250));
        assert_eq!(15, bounded_int_value(Some(15), 10, 1, 250));
        assert_eq!(1, bounded_int_value(Some(-8), 10, 1, 250));
        assert_eq!(250, bounded_int_value(Some(1000), 10, 1, 250));
    }

    #[test]
    fn test_paginate_vector() {
        // sensible "first" page
        let mut input: Vec<u32> = Vec::new();
        for v in 0..102 {
            input.push(v);
        }
        let actual = paginate_vector(&mut input, Some(0), Some(10));
        assert_eq!(actual.len(), 10);
        assert_eq!(actual[0], 0);
        assert_eq!(actual[9], 9);

        // page somewhere in the middle
        let mut input: Vec<u32> = Vec::new();
        for v in 0..102 {
            input.push(v);
        }
        let actual = paginate_vector(&mut input, Some(40), Some(20));
        assert_eq!(actual.len(), 20);
        assert_eq!(actual[0], 40);
        assert_eq!(actual[19], 59);

        // last page with over extension
        let mut input: Vec<u32> = Vec::new();
        for v in 0..102 {
            input.push(v);
        }
        let actual = paginate_vector(&mut input, Some(90), Some(100));
        assert_eq!(actual.len(), 12);
        assert_eq!(actual[0], 90);
        assert_eq!(actual[11], 101);
    }

    #[test]
    fn test_query_asset_ok() {
        // arrange
        let asset1 = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1048576,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cat".to_owned(), "dog".to_owned()],
            import_date: Utc.ymd(2018, 5, 31).and_hms(21, 10, 11),
            caption: None,
            location: Some("hawaii".to_owned()),
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let mut mock = MockEntityDataSource::new();
        mock.expect_get_asset()
            .with(eq("abc123"))
            .returning(move |_| Ok(asset1.clone()));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let query = r#"query Fetch($id: String!) {
            asset(id: $id) {
                id filename filesize datetime mimetype
                tags userdate caption location
            }
        }"#;
        let mut vars = Variables::new();
        vars.insert("id".to_owned(), InputValue::scalar("abc123"));
        let (res, _errors) = juniper::execute(query, None, &schema, &vars, &ctx).unwrap();
        // assert
        let res = res.as_object_value().unwrap();
        let res = res.get_field_value("asset").unwrap();
        let object = res.as_object_value().unwrap();

        let res = object.get_field_value("id").unwrap();
        let actual = res.as_scalar_value::<String>().unwrap();
        assert_eq!(actual, "abc123");

        let res = object.get_field_value("filename").unwrap();
        let actual = res.as_scalar_value::<String>().unwrap();
        assert_eq!(actual, "img_1234.jpg");

        // filesize is a BigInt that is represented as a String
        let res = object.get_field_value("filesize").unwrap();
        let actual = res.as_scalar_value::<String>().unwrap();
        assert_eq!(actual, "1048576");

        // datetime is a DateTime that is represented as a String
        let res = object.get_field_value("datetime").unwrap();
        let actual = res.as_scalar_value::<String>().unwrap();
        assert_eq!(&actual[..19], "2018-05-31T21:10:11");

        let res = object.get_field_value("mimetype").unwrap();
        let actual = res.as_scalar_value::<String>().unwrap();
        assert_eq!(actual, "image/jpeg");

        let res = object.get_field_value("tags").unwrap();
        let list_result = res.as_list_value().unwrap();
        let tags = ["cat", "dog"];
        for (idx, entry) in list_result.iter().enumerate() {
            let actual = entry.as_scalar_value::<String>().unwrap();
            assert_eq!(actual, tags[idx]);
        }

        let res = object.get_field_value("userdate").unwrap();
        assert!(res.is_null());
        let res = object.get_field_value("caption").unwrap();
        assert!(res.is_null());

        let res = object.get_field_value("location").unwrap();
        let actual = res.as_scalar_value::<String>().unwrap();
        assert_eq!(actual, "hawaii");
    }

    #[test]
    fn test_query_asset_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_get_asset()
            .with(eq("abc123"))
            .returning(|_| Err(err_msg("oh no")));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let query = r#"query Fetch($id: String!) {
            asset(id: $id) {
                id filename filesize datetime mimetype
                tags userdate caption location
            }
        }"#;
        let mut vars = Variables::new();
        vars.insert("id".to_owned(), InputValue::scalar("abc123"));
        let (res, errors) = juniper::execute(query, None, &schema, &vars, &ctx).unwrap();
        // assert
        assert!(res.is_null());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].error().message().contains("oh no"));
    }

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
    fn test_query_lookup_some() {
        // arrange
        let asset1 = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1048576,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cat".to_owned(), "dog".to_owned()],
            import_date: Utc.ymd(2018, 5, 31).and_hms(21, 10, 11),
            caption: None,
            location: Some("hawaii".to_owned()),
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_by_checksum()
            .with(eq("cafebabe"))
            .returning(move |_| Ok(Some("abc123".to_owned())));
        mock.expect_get_asset()
            .with(eq("abc123"))
            .returning(move |_| Ok(asset1.clone()));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let query = r#"query Lookup($checksum: String!) {
            lookup(checksum: $checksum) { id }
        }"#;
        let mut vars = Variables::new();
        vars.insert("checksum".to_owned(), InputValue::scalar("cafebabe"));
        let (res, _errors) = juniper::execute(query, None, &schema, &vars, &ctx).unwrap();
        // assert
        let res = res.as_object_value().unwrap();
        let res = res.get_field_value("lookup").unwrap();
        let object = res.as_object_value().unwrap();
        let res = object.get_field_value("id").unwrap();
        let actual = res.as_scalar_value::<String>().unwrap();
        assert_eq!(actual, "abc123");
    }

    #[test]
    fn test_query_lookup_none() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_by_checksum()
            .with(eq("cafebabe"))
            .returning(|_| Ok(None));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let query = r#"query Lookup($checksum: String!) {
            lookup(checksum: $checksum) { id }
        }"#;
        let mut vars = Variables::new();
        vars.insert("checksum".to_owned(), InputValue::scalar("cafebabe"));
        let (res, errors) = juniper::execute(query, None, &schema, &vars, &ctx).unwrap();
        // assert
        let res = res.as_object_value().unwrap();
        let res = res.get_field_value("lookup").unwrap();
        assert!(res.is_null());
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_query_lookup_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_by_checksum()
            .with(eq("cafebabe"))
            .returning(|_| Err(err_msg("oh no")));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let query = r#"query Lookup($checksum: String!) {
            lookup(checksum: $checksum) { id }
        }"#;
        let mut vars = Variables::new();
        vars.insert("checksum".to_owned(), InputValue::scalar("cafebabe"));
        let (res, errors) = juniper::execute(query, None, &schema, &vars, &ctx).unwrap();
        // assert
        let res = res.as_object_value().unwrap();
        let res = res.get_field_value("lookup").unwrap();
        assert!(res.is_null());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].error().message().contains("oh no"));
    }

    fn make_search_results() -> Vec<SearchResult> {
        vec![
            SearchResult {
                asset_id: "cafebabe".to_owned(),
                filename: "img_1234.png".to_owned(),
                media_type: "image/png".to_owned(),
                location: Some("hawaii".to_owned()),
                datetime: Utc.ymd(2012, 5, 31).and_hms(21, 10, 11),
            },
            SearchResult {
                asset_id: "babecafe".to_owned(),
                filename: "img_2345.gif".to_owned(),
                media_type: "image/gif".to_owned(),
                location: Some("london".to_owned()),
                datetime: Utc.ymd(2013, 5, 31).and_hms(21, 10, 11),
            },
            SearchResult {
                asset_id: "cafed00d".to_owned(),
                filename: "img_3456.mov".to_owned(),
                media_type: "video/quicktime".to_owned(),
                location: Some("paris".to_owned()),
                datetime: Utc.ymd(2014, 5, 31).and_hms(21, 10, 11),
            },
            SearchResult {
                asset_id: "d00dcafe".to_owned(),
                filename: "img_4567.jpg".to_owned(),
                media_type: "image/jpeg".to_owned(),
                location: Some("hawaii".to_owned()),
                datetime: Utc.ymd(2015, 5, 31).and_hms(21, 10, 11),
            },
            SearchResult {
                asset_id: "deadbeef".to_owned(),
                filename: "img_5678.mov".to_owned(),
                media_type: "video/quicktime".to_owned(),
                location: Some("london".to_owned()),
                datetime: Utc.ymd(2016, 5, 31).and_hms(21, 10, 11),
            },
            SearchResult {
                asset_id: "cafebeef".to_owned(),
                filename: "img_6789.jpg".to_owned(),
                media_type: "image/jpeg".to_owned(),
                location: Some("paris".to_owned()),
                datetime: Utc.ymd(2017, 5, 31).and_hms(21, 10, 11),
            },
            SearchResult {
                asset_id: "deadcafe".to_owned(),
                filename: "img_7890.jpg".to_owned(),
                media_type: "image/jpeg".to_owned(),
                location: Some("yosemite".to_owned()),
                datetime: Utc.ymd(2018, 5, 31).and_hms(21, 10, 11),
            },
        ]
    }

    #[test]
    fn test_query_search_ok() {
        // arrange
        let results = make_search_results();
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_by_tags()
            .with(always())
            .returning(move |_| Ok(results.clone()));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let mut vars = Variables::new();
        let params = SearchParams {
            tags: Some(vec!["cat".to_owned()]),
            locations: None,
            after: None,
            before: None,
            filename: None,
            mimetype: None,
            sort_field: Some(SortField::Identifier),
            sort_order: Some(SortOrder::Ascending),
        };
        vars.insert("params".to_owned(), params.to_input_value());
        let (res, errors) = juniper::execute(
            r#"query Search($params: SearchParams!) {
                search(params: $params) {
                    results { id filename mimetype location datetime }
                    count
                }
            }"#,
            None,
            &schema,
            &vars,
            &ctx,
        )
        .unwrap();
        // assert
        assert_eq!(errors.len(), 0);
        let res = res.as_object_value().unwrap();
        let res = res.get_field_value("search").unwrap();
        let search = res.as_object_value().unwrap();
        let count_field = search.get_field_value("count").unwrap();
        let count_value = count_field.as_scalar_value::<i32>().unwrap();
        assert_eq!(*count_value, 7);
        let results_field = search.get_field_value("results").unwrap();
        let result_value = results_field.as_list_value().unwrap();

        // check the first result
        let entry_object = result_value[0].as_object_value().unwrap();
        let entry_field = entry_object.get_field_value("id").unwrap();
        let entry_value = entry_field.as_scalar_value::<String>().unwrap();
        assert_eq!(entry_value, "babecafe");
        let entry_field = entry_object.get_field_value("filename").unwrap();
        let entry_value = entry_field.as_scalar_value::<String>().unwrap();
        assert_eq!(entry_value, "img_2345.gif");
        let entry_field = entry_object.get_field_value("mimetype").unwrap();
        let entry_value = entry_field.as_scalar_value::<String>().unwrap();
        assert_eq!(entry_value, "image/gif");
        let entry_field = entry_object.get_field_value("location").unwrap();
        let entry_value = entry_field.as_scalar_value::<String>().unwrap();
        assert_eq!(entry_value, "london");
        let entry_field = entry_object.get_field_value("datetime").unwrap();
        let entry_value = entry_field.as_scalar_value::<String>().unwrap();
        assert_eq!(&entry_value[..19], "2013-05-31T21:10:11");

        // check the last result
        let entry_object = result_value[6].as_object_value().unwrap();
        let entry_field = entry_object.get_field_value("id").unwrap();
        let entry_value = entry_field.as_scalar_value::<String>().unwrap();
        assert_eq!(entry_value, "deadcafe");
        let entry_field = entry_object.get_field_value("filename").unwrap();
        let entry_value = entry_field.as_scalar_value::<String>().unwrap();
        assert_eq!(entry_value, "img_7890.jpg");
        let entry_field = entry_object.get_field_value("mimetype").unwrap();
        let entry_value = entry_field.as_scalar_value::<String>().unwrap();
        assert_eq!(entry_value, "image/jpeg");
        let entry_field = entry_object.get_field_value("location").unwrap();
        let entry_value = entry_field.as_scalar_value::<String>().unwrap();
        assert_eq!(entry_value, "yosemite");
        let entry_field = entry_object.get_field_value("datetime").unwrap();
        let entry_value = entry_field.as_scalar_value::<String>().unwrap();
        assert_eq!(&entry_value[..19], "2018-05-31T21:10:11");
    }

    fn make_many_results() -> Vec<SearchResult> {
        let mut results: Vec<SearchResult> = Vec::new();
        let locations = ["hawaii", "paris", "london"];
        for index in 1..108 {
            // add leading zeros so sorting by id works naturally
            let asset_id = format!("cafebabe-{:04}", index);
            let filename = format!("img_1{}.jpg", index);
            let base_time = Utc.ymd(2012, 5, 31).and_hms(21, 10, 11);
            let duration = chrono::Duration::days(index);
            let datetime = base_time + duration;
            let location_index = (index % locations.len() as i64) as usize;
            results.push(SearchResult {
                asset_id,
                filename,
                media_type: "image/jpeg".to_owned(),
                location: Some(locations[location_index].to_owned()),
                datetime,
            });
        }
        results
    }

    #[test]
    fn test_query_recent_ok() {
        // arrange
        let results = vec![SearchResult {
            asset_id: "cafebabe".to_owned(),
            filename: "img_1234.png".to_owned(),
            media_type: "image/png".to_owned(),
            location: None,
            datetime: Utc.ymd(2019, 5, 13).and_hms(20, 46, 11),
        }];
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_newborn()
            .with(always())
            .returning(move |_| Ok(results.clone()));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let mut vars = Variables::new();
        let since = Some(Utc::now());
        vars.insert("since".to_owned(), since.to_input_value());
        let (res, errors) = juniper::execute(
            r#"query Recent($since: DateTimeUtc) {
                recent(since: $since) {
                    results { id }
                    count
                }
            }"#,
            None,
            &schema,
            &vars,
            &ctx,
        )
        .unwrap();
        // assert
        assert_eq!(errors.len(), 0);
        let res = res.as_object_value().unwrap();
        let res = res.get_field_value("recent").unwrap();
        let recent = res.as_object_value().unwrap();
        let count_field = recent.get_field_value("count").unwrap();
        let count_value = count_field.as_scalar_value::<i32>().unwrap();
        assert_eq!(*count_value, 1);
        let results_field = recent.get_field_value("results").unwrap();
        let result_value = results_field.as_list_value().unwrap();
        assert_eq!(result_value.len(), 1);

        // check the result
        let entry_object = result_value[0].as_object_value().unwrap();
        let entry_field = entry_object.get_field_value("id").unwrap();
        let entry_value = entry_field.as_scalar_value::<String>().unwrap();
        assert_eq!(entry_value, "cafebabe");
    }

    #[test]
    fn test_query_recent_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_newborn()
            .with(always())
            .returning(|_| Err(err_msg("oh no")));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let mut vars = Variables::new();
        let since = Some(Utc::now());
        vars.insert("since".to_owned(), since.to_input_value());
        let (res, errors) = juniper::execute(
            r#"query Recent($since: DateTimeUtc) {
                recent(since: $since) {
                    results { id }
                    count
                }
            }"#,
            None,
            &schema,
            &vars,
            &ctx,
        )
        .unwrap();
        // assert
        assert!(res.is_null());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].error().message().contains("oh no"));
    }

    #[test]
    fn test_query_search_page_first() {
        // arrange
        let results = make_many_results();
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_by_tags()
            .with(always())
            .returning(move |_| Ok(results.clone()));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let mut vars = Variables::new();
        let params = SearchParams {
            tags: Some(vec!["cat".to_owned()]),
            locations: None,
            after: None,
            before: None,
            filename: None,
            mimetype: None,
            sort_field: Some(SortField::Identifier),
            sort_order: Some(SortOrder::Ascending),
        };
        vars.insert("params".to_owned(), params.to_input_value());
        let (res, errors) = juniper::execute(
            r#"query Search($params: SearchParams!) {
                search(params: $params, count: 10) {
                    results { id }
                    count
                }
            }"#,
            None,
            &schema,
            &vars,
            &ctx,
        )
        .unwrap();
        // assert
        assert_eq!(errors.len(), 0);
        let res = res.as_object_value().unwrap();
        let res = res.get_field_value("search").unwrap();
        let search = res.as_object_value().unwrap();
        let count_field = search.get_field_value("count").unwrap();
        let count_value = count_field.as_scalar_value::<i32>().unwrap();
        assert_eq!(*count_value, 107);
        let results_field = search.get_field_value("results").unwrap();
        let result_value = results_field.as_list_value().unwrap();
        assert_eq!(result_value.len(), 10);

        // check the first result
        let entry_object = result_value[0].as_object_value().unwrap();
        let entry_field = entry_object.get_field_value("id").unwrap();
        let entry_value = entry_field.as_scalar_value::<String>().unwrap();
        assert_eq!(entry_value, "cafebabe-0001");

        // check the last result
        let entry_object = result_value[9].as_object_value().unwrap();
        let entry_field = entry_object.get_field_value("id").unwrap();
        let entry_value = entry_field.as_scalar_value::<String>().unwrap();
        assert_eq!(entry_value, "cafebabe-0010");
    }

    #[test]
    fn test_query_search_page_middle() {
        // arrange
        let results = make_many_results();
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_by_tags()
            .with(always())
            .returning(move |_| Ok(results.clone()));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let mut vars = Variables::new();
        let params = SearchParams {
            tags: Some(vec!["cat".to_owned()]),
            locations: None,
            after: None,
            before: None,
            filename: None,
            mimetype: None,
            sort_field: Some(SortField::Identifier),
            sort_order: Some(SortOrder::Ascending),
        };
        vars.insert("params".to_owned(), params.to_input_value());
        let (res, errors) = juniper::execute(
            r#"query Search($params: SearchParams!) {
                search(params: $params, count: 10, offset: 20) {
                    results { id }
                    count
                }
            }"#,
            None,
            &schema,
            &vars,
            &ctx,
        )
        .unwrap();
        // assert
        assert_eq!(errors.len(), 0);
        let res = res.as_object_value().unwrap();
        let res = res.get_field_value("search").unwrap();
        let search = res.as_object_value().unwrap();
        let count_field = search.get_field_value("count").unwrap();
        let count_value = count_field.as_scalar_value::<i32>().unwrap();
        assert_eq!(*count_value, 107);
        let results_field = search.get_field_value("results").unwrap();
        let result_value = results_field.as_list_value().unwrap();
        assert_eq!(result_value.len(), 10);

        // check the first result
        let entry_object = result_value[0].as_object_value().unwrap();
        let entry_field = entry_object.get_field_value("id").unwrap();
        let entry_value = entry_field.as_scalar_value::<String>().unwrap();
        assert_eq!(entry_value, "cafebabe-0021");

        // check the last result
        let entry_object = result_value[9].as_object_value().unwrap();
        let entry_field = entry_object.get_field_value("id").unwrap();
        let entry_value = entry_field.as_scalar_value::<String>().unwrap();
        assert_eq!(entry_value, "cafebabe-0030");
    }

    #[test]
    fn test_query_search_page_last() {
        // arrange
        let results = make_many_results();
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_by_tags()
            .with(always())
            .returning(move |_| Ok(results.clone()));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let mut vars = Variables::new();
        let params = SearchParams {
            tags: Some(vec!["cat".to_owned()]),
            locations: None,
            after: None,
            before: None,
            filename: None,
            mimetype: None,
            sort_field: Some(SortField::Identifier),
            sort_order: Some(SortOrder::Ascending),
        };
        vars.insert("params".to_owned(), params.to_input_value());
        let (res, errors) = juniper::execute(
            r#"query Search($params: SearchParams!) {
                search(params: $params, count: 100, offset: 80) {
                    results { id }
                    count
                }
            }"#,
            None,
            &schema,
            &vars,
            &ctx,
        )
        .unwrap();
        // assert
        assert_eq!(errors.len(), 0);
        let res = res.as_object_value().unwrap();
        let res = res.get_field_value("search").unwrap();
        let search = res.as_object_value().unwrap();
        let count_field = search.get_field_value("count").unwrap();
        let count_value = count_field.as_scalar_value::<i32>().unwrap();
        assert_eq!(*count_value, 107);
        let results_field = search.get_field_value("results").unwrap();
        let result_value = results_field.as_list_value().unwrap();
        assert_eq!(result_value.len(), 27);

        // check the first result
        let entry_object = result_value[0].as_object_value().unwrap();
        let entry_field = entry_object.get_field_value("id").unwrap();
        let entry_value = entry_field.as_scalar_value::<String>().unwrap();
        assert_eq!(entry_value, "cafebabe-0081");

        // check the last result
        let entry_object = result_value[26].as_object_value().unwrap();
        let entry_field = entry_object.get_field_value("id").unwrap();
        let entry_value = entry_field.as_scalar_value::<String>().unwrap();
        assert_eq!(entry_value, "cafebabe-0107");
    }

    #[test]
    fn test_query_search_complex() {
        // arrange
        let results = make_many_results();
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_by_tags()
            .with(always())
            .returning(move |_| Ok(results.clone()));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let mut vars = Variables::new();
        // slightly more complex search parameters
        let params = SearchParams {
            tags: Some(vec!["cat".to_owned()]),
            locations: Some(vec!["hawaii".to_owned()]),
            after: None,
            before: None,
            filename: None,
            mimetype: None,
            sort_field: Some(SortField::Identifier),
            sort_order: Some(SortOrder::Ascending),
        };
        vars.insert("params".to_owned(), params.to_input_value());
        let (res, errors) = juniper::execute(
            r#"query Search($params: SearchParams!) {
                search(params: $params, count: 100) {
                    results { id }
                    count
                }
            }"#,
            None,
            &schema,
            &vars,
            &ctx,
        )
        .unwrap();
        // assert
        assert_eq!(errors.len(), 0);
        let res = res.as_object_value().unwrap();
        let res = res.get_field_value("search").unwrap();
        let search = res.as_object_value().unwrap();
        let count_field = search.get_field_value("count").unwrap();
        let count_value = count_field.as_scalar_value::<i32>().unwrap();
        assert_eq!(*count_value, 35);
        let results_field = search.get_field_value("results").unwrap();
        let result_value = results_field.as_list_value().unwrap();
        assert_eq!(result_value.len(), 35);

        // check the first result
        let entry_object = result_value[0].as_object_value().unwrap();
        let entry_field = entry_object.get_field_value("id").unwrap();
        let entry_value = entry_field.as_scalar_value::<String>().unwrap();
        assert_eq!(entry_value, "cafebabe-0003");

        // check the last result
        let entry_object = result_value[34].as_object_value().unwrap();
        let entry_field = entry_object.get_field_value("id").unwrap();
        let entry_value = entry_field.as_scalar_value::<String>().unwrap();
        assert_eq!(entry_value, "cafebabe-0105");
    }

    #[test]
    fn test_query_search_none() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_by_tags()
            .with(always())
            .returning(move |_| Ok(vec![]));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let mut vars = Variables::new();
        let params = SearchParams {
            tags: Some(vec!["cat".to_owned()]),
            locations: None,
            after: None,
            before: None,
            filename: None,
            mimetype: None,
            sort_field: Some(SortField::Identifier),
            sort_order: Some(SortOrder::Ascending),
        };
        vars.insert("params".to_owned(), params.to_input_value());
        let (res, errors) = juniper::execute(
            r#"query Search($params: SearchParams!) {
                search(params: $params) {
                    results { id filename mimetype location datetime }
                    count
                }
            }"#,
            None,
            &schema,
            &vars,
            &ctx,
        )
        .unwrap();
        // assert
        assert_eq!(errors.len(), 0);
        let res = res.as_object_value().unwrap();
        let res = res.get_field_value("search").unwrap();
        let search = res.as_object_value().unwrap();
        let count_field = search.get_field_value("count").unwrap();
        let count_value = count_field.as_scalar_value::<i32>().unwrap();
        assert_eq!(*count_value, 0);
        let results_field = search.get_field_value("results").unwrap();
        let result_value = results_field.as_list_value().unwrap();
        assert_eq!(result_value.len(), 0);
    }

    #[test]
    fn test_query_search_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_by_tags()
            .with(eq(vec!["cat".to_owned()]))
            .returning(|_| Err(err_msg("oh no")));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let mut vars = Variables::new();
        let params = SearchParams {
            tags: Some(vec!["cat".to_owned()]),
            locations: None,
            after: None,
            before: None,
            filename: None,
            mimetype: None,
            sort_field: Some(SortField::Identifier),
            sort_order: Some(SortOrder::Ascending),
        };
        vars.insert("params".to_owned(), params.to_input_value());
        let (res, errors) = juniper::execute(
            r#"query Search($params: SearchParams!) {
                search(params: $params) {
                    results { id }
                    count
                }
            }"#,
            None,
            &schema,
            &vars,
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

    #[test]
    fn test_mutation_update_ok() {
        // arrange
        let asset1 = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1048576,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cat".to_owned(), "dog".to_owned()],
            import_date: Utc.ymd(2018, 5, 31).and_hms(21, 10, 11),
            caption: None,
            location: Some("hawaii".to_owned()),
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let mut mock = MockEntityDataSource::new();
        mock.expect_get_asset()
            .with(eq("abc123"))
            .returning(move |_| Ok(asset1.clone()));
        mock.expect_put_asset().with(always()).returning(|_| Ok(()));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let mut vars = Variables::new();
        let input = AssetInput {
            tags: Some(vec!["kitten".to_owned()]),
            caption: Some("saw a #cat playing".to_owned()),
            location: Some("london".to_owned()),
            datetime: None,
            mimetype: None,
        };
        vars.insert("input".to_owned(), input.to_input_value());
        let (res, errors) = juniper::execute(
            r#"mutation Update($input: AssetInput!) {
                update(id: "abc123", asset: $input) {
                    id tags location caption
                }
            }"#,
            None,
            &schema,
            &vars,
            &ctx,
        )
        .unwrap();
        // assert
        assert_eq!(errors.len(), 0);
        let res = res.as_object_value().unwrap();
        let res = res.get_field_value("update").unwrap();
        let object = res.as_object_value().unwrap();
        let field = object.get_field_value("id").unwrap();
        let value = field.as_scalar_value::<String>().unwrap();
        assert_eq!(value, "abc123");
        let field = object.get_field_value("location").unwrap();
        let value = field.as_scalar_value::<String>().unwrap();
        assert_eq!(value, "london");
        let field = object.get_field_value("caption").unwrap();
        let value = field.as_scalar_value::<String>().unwrap();
        assert_eq!(value, "saw a #cat playing");
        let field = object.get_field_value("tags").unwrap();
        let value = field.as_list_value().unwrap();
        let tags = ["cat", "kitten"];
        for (idx, entry) in value.iter().enumerate() {
            let actual = entry.as_scalar_value::<String>().unwrap();
            assert_eq!(actual, tags[idx]);
        }
    }

    #[test]
    fn test_mutation_update_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_get_asset()
            .with(always())
            .returning(|_| Err(err_msg("oh no")));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let mut vars = Variables::new();
        let input = AssetInput {
            tags: Some(vec!["kitten".to_owned()]),
            caption: None,
            location: None,
            datetime: None,
            mimetype: None,
        };
        vars.insert("input".to_owned(), input.to_input_value());
        let (res, errors) = juniper::execute(
            r#"mutation Update($input: AssetInput!) {
                update(id: "abc123", asset: $input) { id }
            }"#,
            None,
            &schema,
            &vars,
            &ctx,
        )
        .unwrap();
        // assert
        assert!(res.is_null());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].error().message().contains("oh no"));
    }

    #[test]
    fn test_update_empty_location() {
        // arrange
        let asset1 = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1048576,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cat".to_owned(), "dog".to_owned()],
            import_date: Utc.ymd(2018, 5, 31).and_hms(21, 10, 11),
            caption: None,
            location: Some("hawaii".to_owned()),
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let mut mock = MockEntityDataSource::new();
        mock.expect_get_asset()
            .with(eq("abc123"))
            .returning(move |_| Ok(asset1.clone()));
        mock.expect_put_asset().with(always()).returning(|_| Ok(()));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let mut vars = Variables::new();
        let input = AssetInput {
            tags: Some(vec!["kitten".to_owned()]),
            caption: Some("saw a #cat playing".to_owned()),
            location: Some("".to_owned()),
            datetime: None,
            mimetype: None,
        };
        vars.insert("input".to_owned(), input.to_input_value());
        let (res, errors) = juniper::execute(
            r#"mutation Update($input: AssetInput!) {
                update(id: "abc123", asset: $input) {
                    id tags location caption
                }
            }"#,
            None,
            &schema,
            &vars,
            &ctx,
        )
        .unwrap();
        // assert
        assert_eq!(errors.len(), 0);
        let res = res.as_object_value().unwrap();
        let res = res.get_field_value("update").unwrap();
        let object = res.as_object_value().unwrap();
        let field = object.get_field_value("location").unwrap();
        assert!(field.is_null());
    }

    #[test]
    fn test_mutation_bulk_update_ok() {
        // arrange
        let asset1 = Asset {
            key: "monday6".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1048576,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cat".to_owned(), "dog".to_owned()],
            import_date: Utc.ymd(2018, 5, 31).and_hms(21, 10, 11),
            caption: None,
            location: Some("hawaii".to_owned()),
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let asset2 = Asset {
            key: "tuesday7".to_owned(),
            checksum: "cafed00d".to_owned(),
            filename: "img_2468.jpg".to_owned(),
            byte_length: 1048576,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cat".to_owned(), "dog".to_owned()],
            import_date: Utc.ymd(2018, 6, 9).and_hms(14, 0, 11),
            caption: None,
            location: Some("oakland".to_owned()),
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let mut mock = MockEntityDataSource::new();
        mock.expect_get_asset()
            .with(eq("monday6"))
            .returning(move |_| Ok(asset1.clone()));
        mock.expect_get_asset()
            .with(eq("tuesday7"))
            .returning(move |_| Ok(asset2.clone()));
        mock.expect_put_asset().with(always()).returning(|_| Ok(()));
        let ctx: Arc<dyn EntityDataSource> = Arc::new(mock);
        // act
        let schema = create_schema();
        let mut vars = Variables::new();
        let assets = vec![
            AssetInputId {
                id: "monday6".to_owned(),
                input: AssetInput {
                    tags: Some(vec!["kitten".to_owned()]),
                    caption: Some("saw a #cat playing with a #dog".to_owned()),
                    location: Some("hawaii".to_owned()),
                    datetime: None,
                    mimetype: None,
                },
            },
            AssetInputId {
                id: "tuesday7".to_owned(),
                input: AssetInput {
                    tags: Some(vec!["kitten".to_owned()]),
                    caption: Some("saw a #cat playing".to_owned()),
                    location: Some("london".to_owned()),
                    datetime: None,
                    mimetype: None,
                },
            },
        ];
        vars.insert("assets".to_owned(), assets.to_input_value());
        let (res, errors) = juniper::execute(
            r#"mutation BulkUpdate($assets: [AssetInputId!]!) {
                bulkUpdate(assets: $assets)
            }"#,
            None,
            &schema,
            &vars,
            &ctx,
        )
        .unwrap();
        // assert
        assert_eq!(errors.len(), 0);
        let res = res.as_object_value().unwrap();
        let res = res.get_field_value("bulkUpdate").unwrap();
        let actual = res.as_scalar_value::<i32>().unwrap();
        assert_eq!(*actual, 2);
    }
}
