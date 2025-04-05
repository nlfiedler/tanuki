//
// Copyright (c) 2025 Nathan Fiedler
//
use crate::data::repositories::geo::find_location_repository;
use crate::data::repositories::{BlobRepositoryImpl, RecordRepositoryImpl, SearchRepositoryImpl};
use crate::data::sources::EntityDataSource;
use crate::domain::entities::{Asset, LabeledCount, Location};
use crate::domain::usecases::analyze::Counts;
use crate::domain::usecases::diagnose::Diagnosis;
use chrono::prelude::*;
use juniper::{
    EmptySubscription, FieldResult, GraphQLEnum, GraphQLScalar, InputValue, ParseScalarResult,
    ParseScalarValue, RootNode, ScalarToken, ScalarValue, Value,
};
use log::error;
use std::path::PathBuf;
use std::sync::Arc;

// Context for the GraphQL schema.
pub struct GraphContext {
    datasource: Arc<dyn EntityDataSource>,
    assets_path: Box<PathBuf>,
}

impl GraphContext {
    pub fn new(datasource: Arc<dyn EntityDataSource>, assets_path: Box<PathBuf>) -> Self {
        Self {
            datasource,
            assets_path,
        }
    }
}

// Mark the data source as a valid context type for Juniper.
impl juniper::Context for GraphContext {}

/// An integer type larger than the standard signed 32-bit.
#[derive(Copy, Clone, Debug, Eq, GraphQLScalar, PartialEq)]
#[graphql(with = Self)]
pub struct BigInt(i64);

impl BigInt {
    /// Construct a BigInt for the given value.
    pub fn new(value: i64) -> Self {
        BigInt(value)
    }

    fn to_output<S: ScalarValue>(&self) -> Value<S> {
        Value::scalar(format!("{}", self.0))
    }

    fn from_input<S: ScalarValue>(v: &InputValue<S>) -> Result<Self, String> {
        v.as_scalar_value()
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .map(BigInt)
            .ok_or_else(|| format!("Expected `BigInt`, found: {v}"))
    }

    fn parse_token<S: ScalarValue>(value: ScalarToken<'_>) -> ParseScalarResult<S> {
        <String as ParseScalarValue<S>>::from_str(value)
    }
}

impl From<BigInt> for u32 {
    fn from(val: BigInt) -> Self {
        val.0 as u32
    }
}

impl From<BigInt> for u64 {
    fn from(val: BigInt) -> Self {
        val.0 as u64
    }
}

impl From<u32> for BigInt {
    fn from(t: u32) -> Self {
        BigInt(i64::from(t))
    }
}

#[juniper::graphql_object(description = "Information regarding a location for an asset.")]
impl Location {
    /// Label for the location provided by the user.
    fn label(&self) -> Option<String> {
        self.label.clone()
    }

    /// Name of the city of which the asset is associated.
    fn city(&self) -> Option<String> {
        self.city.clone()
    }

    /// Name of the state or province of which the asset is associated.
    fn region(&self) -> Option<String> {
        self.region.clone()
    }
}

#[juniper::graphql_object(
    description = "An `Asset` defines a single entity in the storage system."
)]
impl Asset {
    /// The unique asset identifier.
    fn id(&self) -> String {
        self.key.clone()
    }

    /// Hash digest of the contents of the asset.
    #[graphql(name = "checksum")]
    fn checksum_gql(&self) -> String {
        self.checksum.clone()
    }

    /// The original filename of the asset when it was imported.
    #[graphql(name = "filename")]
    fn filename_gql(&self) -> String {
        self.filename.clone()
    }

    /// The size in bytes of the asset.
    fn filesize(&self) -> BigInt {
        BigInt(self.byte_length as i64)
    }

    /// Relative path of the asset within the asset store.
    #[graphql(name = "filepath")]
    fn filepath_gql(&self) -> String {
        self.filepath()
    }

    /// The date/time that best represents the asset.
    fn datetime(&self) -> DateTime<Utc> {
        if let Some(ud) = self.user_date.as_ref() {
            *ud
        } else if let Some(od) = self.original_date.as_ref() {
            *od
        } else {
            self.import_date
        }
    }

    /// The media type of the asset, such as "image/jpeg" or "video/mp4".
    #[graphql(name = "mediaType")]
    fn media_type_gql(&self) -> String {
        self.media_type.clone()
    }

    /// The list of tags associated with this asset.
    #[graphql(name = "tags")]
    fn tags_gql(&self) -> Vec<String> {
        self.tags.clone()
    }

    /// The date provided by the user.
    fn userdate(&self) -> Option<DateTime<Utc>> {
        self.user_date
    }

    /// A caption attributed to the asset.
    #[graphql(name = "caption")]
    fn caption_gql(&self) -> Option<String> {
        self.caption.clone()
    }

    /// Location information for the asset.
    #[graphql(name = "location")]
    fn location_gql(&self) -> Option<Location> {
        self.location.clone()
    }
}

#[juniper::graphql_object(
    description = "An attribute name and the number of assets it references."
)]
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

//
// Note on using juniper and defining input objects: If a runtime error occurs
// that says "Can't unify non-input concrete type" then try naming the input
// type with a unique name. For instance, trying to define a `Location` input
// object when also implementing the output object for that same-named type from
// the entities module results in this error. It may have something to do with
// the reliance on macros to generate code for interfacing with juniper.
//

#[derive(GraphQLEnum)]
pub enum ErrorCode {
    /// Asset identifier was seemingly not valid base64.
    Base64,
    /// Asset identifier was seemingly not valid UTF-8.
    Utf8,
    /// Asset file was not found at the expected location.
    Missing,
    /// Asset file size does not match database record.
    Size,
    /// Asset file was probably inaccessible (file permissions).
    Access,
    /// Asset file hash digest does not match database record.
    Digest,
    /// Asset record media_type property is not a valid media type.
    MediaType,
    /// Asset record original_date property missing or incorrect.
    OriginalDate,
    /// Missing original filename.
    Filename,
    /// Asset identifier/filename extension missing or incorrect.
    Extension,
    /// Asset file appears to have a different extension than expected.
    Renamed,
}

impl From<crate::domain::usecases::diagnose::ErrorCode> for ErrorCode {
    fn from(code: crate::domain::usecases::diagnose::ErrorCode) -> Self {
        match code {
            crate::domain::usecases::diagnose::ErrorCode::Base64 => ErrorCode::Base64,
            crate::domain::usecases::diagnose::ErrorCode::Utf8 => ErrorCode::Utf8,
            crate::domain::usecases::diagnose::ErrorCode::Missing => ErrorCode::Missing,
            crate::domain::usecases::diagnose::ErrorCode::Size => ErrorCode::Size,
            crate::domain::usecases::diagnose::ErrorCode::Access => ErrorCode::Access,
            crate::domain::usecases::diagnose::ErrorCode::Digest => ErrorCode::Digest,
            crate::domain::usecases::diagnose::ErrorCode::MediaType => ErrorCode::MediaType,
            crate::domain::usecases::diagnose::ErrorCode::OriginalDate => ErrorCode::OriginalDate,
            crate::domain::usecases::diagnose::ErrorCode::Filename => ErrorCode::Filename,
            crate::domain::usecases::diagnose::ErrorCode::Extension => ErrorCode::Extension,
            crate::domain::usecases::diagnose::ErrorCode::Renamed => ErrorCode::Renamed,
        }
    }
}

#[juniper::graphql_object(description = "`Diagnosis` is returned from the `diagnose` query.")]
impl Diagnosis {
    /// Identifier for the asset.
    fn asset_id(&self) -> String {
        self.asset_id.clone()
    }

    /// One of the issues found with this asset.
    fn error_code(&self) -> ErrorCode {
        self.error_code.into()
    }
}

#[juniper::graphql_object(description = "`Counts` is returned from the `analyze` query.")]
impl Counts {
    /// Total number of assets in the database.
    fn total_assets(&self) -> i32 {
        self.total_assets.try_into().unwrap_or(-1)
    }

    /// Number of assets for which the file is missing.
    fn missing_files(&self) -> i32 {
        self.missing_files.try_into().unwrap_or(-1)
    }

    /// Number of assets that represent an image.
    fn is_image(&self) -> i32 {
        self.is_image.try_into().unwrap_or(-1)
    }

    /// Number of assets that represent a video.
    fn is_video(&self) -> i32 {
        self.is_video.try_into().unwrap_or(-1)
    }

    /// Number of images that have Exif data.
    fn has_exif_data(&self) -> i32 {
        self.has_exif_data.try_into().unwrap_or(-1)
    }

    /// Number images that have GPS coordinates.
    fn has_gps_coords(&self) -> i32 {
        self.has_gps_coords.try_into().unwrap_or(-1)
    }

    /// Number images that have an original date/time.
    fn has_original_datetime(&self) -> i32 {
        self.has_original_datetime.try_into().unwrap_or(-1)
    }

    /// Number images that have an original time zone.
    fn has_original_timezone(&self) -> i32 {
        self.has_original_timezone.try_into().unwrap_or(-1)
    }
}

pub struct QueryRoot;

#[juniper::graphql_object(Context = GraphContext)]
impl QueryRoot {
    /// Perform an analysis of the assets and the data they contain.
    fn analyze(#[graphql(ctx)] ctx: &GraphContext) -> FieldResult<Counts> {
        use crate::domain::usecases::analyze::Analyze;
        use crate::domain::usecases::{NoParams, UseCase};
        let repo = RecordRepositoryImpl::new(ctx.datasource.clone());
        let blobs = BlobRepositoryImpl::new(&ctx.assets_path);
        let usecase = Analyze::new(Box::new(repo), Box::new(blobs));
        let counts: Counts = usecase.call(NoParams {})?;
        Ok(counts)
    }

    /// Retrieve an asset by its unique identifier.
    fn asset(#[graphql(ctx)] ctx: &GraphContext, id: String) -> FieldResult<Asset> {
        use crate::domain::usecases::fetch::{FetchAsset, Params};
        use crate::domain::usecases::UseCase;
        let repo = RecordRepositoryImpl::new(ctx.datasource.clone());
        let usecase = FetchAsset::new(Box::new(repo));
        let params = Params::new(id);
        let asset = usecase.call(params)?;
        Ok(asset)
    }

    /// Return the total number of assets in the system.
    fn count(#[graphql(ctx)] ctx: &GraphContext) -> FieldResult<i32> {
        use crate::domain::usecases::count::CountAssets;
        use crate::domain::usecases::{NoParams, UseCase};
        let repo = RecordRepositoryImpl::new(ctx.datasource.clone());
        let usecase = CountAssets::new(Box::new(repo));
        let params = NoParams {};
        let count = usecase.call(params)?;
        Ok(count as i32)
    }

    /// Perform a diagnosis of the database and blob store.
    fn diagnose(
        #[graphql(ctx)] ctx: &GraphContext,
        checksum: Option<bool>,
    ) -> FieldResult<Vec<Diagnosis>> {
        use crate::domain::usecases::diagnose::{Diagnose, Params};
        use crate::domain::usecases::UseCase;
        let repo = RecordRepositoryImpl::new(ctx.datasource.clone());
        let blobs = BlobRepositoryImpl::new(&ctx.assets_path);
        let usecase = Diagnose::new(Box::new(repo), Box::new(blobs));
        let mut params: Params = Default::default();
        if let Some(chk) = checksum {
            params.checksum = chk;
        }
        let results: Vec<Diagnosis> = usecase.call(params)?;
        Ok(results)
    }

    /// Retrieve the list of unique locations with their full structure.
    fn locations(#[graphql(ctx)] ctx: &GraphContext) -> FieldResult<Vec<Location>> {
        use crate::domain::usecases::location::CompleteLocations;
        use crate::domain::usecases::{NoParams, UseCase};
        let repo = RecordRepositoryImpl::new(ctx.datasource.clone());
        let usecase = CompleteLocations::new(Box::new(repo));
        let locations: Vec<Location> = usecase.call(NoParams {})?;
        Ok(locations)
    }

    /// Look for an asset by the hash digest (SHA256).
    fn lookup(#[graphql(ctx)] ctx: &GraphContext, checksum: String) -> FieldResult<Option<Asset>> {
        use crate::domain::usecases::checksum::{AssetByChecksum, Params};
        use crate::domain::usecases::UseCase;
        let repo = RecordRepositoryImpl::new(ctx.datasource.clone());
        let usecase = AssetByChecksum::new(Box::new(repo));
        let params = Params::new(checksum);
        let asset = usecase.call(params)?;
        Ok(asset)
    }

    /// Retrieve the list of media types and their associated asset count.
    fn media_types(#[graphql(ctx)] ctx: &GraphContext) -> FieldResult<Vec<LabeledCount>> {
        use crate::domain::usecases::types::AllMediaTypes;
        use crate::domain::usecases::{NoParams, UseCase};
        let repo = RecordRepositoryImpl::new(ctx.datasource.clone());
        let usecase = AllMediaTypes::new(Box::new(repo));
        let params = NoParams {};
        let types: Vec<LabeledCount> = usecase.call(params)?;
        Ok(types)
    }

    /// Retrieve the list of tags and their associated asset count.
    fn tags(#[graphql(ctx)] ctx: &GraphContext) -> FieldResult<Vec<LabeledCount>> {
        use crate::domain::usecases::tags::AllTags;
        use crate::domain::usecases::{NoParams, UseCase};
        let repo = RecordRepositoryImpl::new(ctx.datasource.clone());
        let usecase = AllTags::new(Box::new(repo));
        let params = NoParams {};
        let tags: Vec<LabeledCount> = usecase.call(params)?;
        Ok(tags)
    }

    /// Retrieve the list of years and their associated asset count.
    fn years(#[graphql(ctx)] ctx: &GraphContext) -> FieldResult<Vec<LabeledCount>> {
        use crate::domain::usecases::year::AllYears;
        use crate::domain::usecases::{NoParams, UseCase};
        let repo = RecordRepositoryImpl::new(ctx.datasource.clone());
        let usecase = AllYears::new(Box::new(repo));
        let params = NoParams {};
        let years: Vec<LabeledCount> = usecase.call(params)?;
        Ok(years)
    }
}

pub struct MutationRoot;

#[juniper::graphql_object(Context = GraphContext)]
impl MutationRoot {
    /// Diagnosis and repair issues in the database and blob store.
    fn repair(
        #[graphql(ctx)] ctx: &GraphContext,
        checksum: Option<bool>,
    ) -> FieldResult<Vec<Diagnosis>> {
        use crate::domain::usecases::diagnose::{Diagnose, Params};
        use crate::domain::usecases::UseCase;
        let repo = RecordRepositoryImpl::new(ctx.datasource.clone());
        let blobs = BlobRepositoryImpl::new(&ctx.assets_path);
        let usecase = Diagnose::new(Box::new(repo), Box::new(blobs));
        let mut params: Params = Default::default();
        if let Some(chk) = checksum {
            params.checksum = chk;
        }
        params.repair = true;
        let results: Vec<Diagnosis> = usecase.call(params)?;
        Ok(results)
    }

    /// Attempt to fill in the city and region for assets that have GPS
    /// coordinates available in the file metadata.
    ///
    /// If overwrite is true, will replace whatever city and region may already
    /// be present.
    fn geocode(#[graphql(ctx)] ctx: &GraphContext, overwrite: bool) -> FieldResult<i32> {
        use crate::domain::usecases::geocode::{Geocoder, Params};
        use crate::domain::usecases::UseCase;
        let repo = RecordRepositoryImpl::new(ctx.datasource.clone());
        let blobs = BlobRepositoryImpl::new(&ctx.assets_path);
        let geocoder = find_location_repository();
        let cache = SearchRepositoryImpl::new();
        let usecase = Geocoder::new(Box::new(repo), Box::new(blobs), geocoder, Box::new(cache));
        match usecase.call(Params::new(overwrite)) {
            Ok(count) => Ok(count as i32),
            Err(err) => {
                error!("graphql: geocode error: {}", err);
                Ok(-1)
            }
        }
    }

    /// Dump all asset records from the database to the given file path in JSON format.
    fn dump(#[graphql(ctx)] ctx: &GraphContext, filepath: String) -> FieldResult<i32> {
        use crate::domain::usecases::dump::{Dump, Params};
        use crate::domain::usecases::UseCase;
        let repo = RecordRepositoryImpl::new(ctx.datasource.clone());
        let usecase = Dump::new(Box::new(repo));
        let params = Params::new(filepath.into());
        let results: u64 = usecase.call(params)?;
        Ok(results as i32)
    }

    /// Load the JSON formatted asset records into the database from the given file path.
    fn load(#[graphql(ctx)] ctx: &GraphContext, filepath: String) -> FieldResult<i32> {
        use crate::domain::usecases::load::{Load, Params};
        use crate::domain::usecases::UseCase;
        let repo = RecordRepositoryImpl::new(ctx.datasource.clone());
        let cache = SearchRepositoryImpl::new();
        let usecase = Load::new(Box::new(repo), Box::new(cache));
        let params = Params::new(filepath.into());
        let results: u64 = usecase.call(params)?;
        Ok(results as i32)
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription<GraphContext>>;

/// Create the GraphQL schema.
pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {}, EmptySubscription::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::sources::MockEntityDataSource;
    use anyhow::anyhow;
    use juniper::{InputValue, Variables};
    use mockall::predicate::*;

    fn make_date_time(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, hour, minute, second)
            .single()
            .unwrap()
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
            import_date: make_date_time(2018, 5, 31, 21, 10, 11),
            caption: None,
            location: Some(Location::new("hawaii")),
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let mut mock = MockEntityDataSource::new();
        mock.expect_get_asset_by_id()
            .with(eq("abc123"))
            .returning(move |_| Ok(asset1.clone()));
        let datasource: Arc<dyn EntityDataSource> = Arc::new(mock);
        let assets_path = Box::new(PathBuf::from("/tmp"));
        let ctx = Arc::new(GraphContext::new(datasource, assets_path));
        // act
        let schema = create_schema();
        let query = r#"query Fetch($id: String!) {
            asset(id: $id) {
                id filename filesize datetime mediaType
                tags userdate caption location { label }
            }
        }"#;
        let mut vars = Variables::new();
        vars.insert("id".to_owned(), InputValue::scalar("abc123"));
        let (res, _errors) = juniper::execute_sync(query, None, &schema, &vars, &ctx).unwrap();
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

        let res = object.get_field_value("mediaType").unwrap();
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
        let object = res.as_object_value().unwrap();
        let res = object.get_field_value("label").unwrap();
        let actual = res.as_scalar_value::<String>().unwrap();
        assert_eq!(actual, "hawaii");
    }

    #[test]
    fn test_query_asset_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_get_asset_by_id()
            .with(eq("abc123"))
            .returning(|_| Err(anyhow!("oh no")));
        let datasource: Arc<dyn EntityDataSource> = Arc::new(mock);
        let assets_path = Box::new(PathBuf::from("/tmp"));
        let ctx = Arc::new(GraphContext::new(datasource, assets_path));
        // act
        let schema = create_schema();
        let query = r#"query Fetch($id: String!) {
            asset(id: $id) {
                id filename filesize datetime mediaType
                tags userdate caption location { label }
            }
        }"#;
        let mut vars = Variables::new();
        vars.insert("id".to_owned(), InputValue::scalar("abc123"));
        let (res, errors) = juniper::execute_sync(query, None, &schema, &vars, &ctx).unwrap();
        // assert
        assert!(res.is_null());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].error().message().contains("oh no"));
    }

    #[test]
    fn test_query_count_ok() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_count_assets().returning(|| Ok(42));
        let datasource: Arc<dyn EntityDataSource> = Arc::new(mock);
        let assets_path = Box::new(PathBuf::from("/tmp"));
        let ctx = Arc::new(GraphContext::new(datasource, assets_path));
        // act
        let schema = create_schema();
        let (res, _errors) =
            juniper::execute_sync(r#"query { count }"#, None, &schema, &Variables::new(), &ctx)
                .unwrap();
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
            .returning(|| Err(anyhow!("oh no")));
        let datasource: Arc<dyn EntityDataSource> = Arc::new(mock);
        let assets_path = Box::new(PathBuf::from("/tmp"));
        let ctx = Arc::new(GraphContext::new(datasource, assets_path));
        // act
        let schema = create_schema();
        let (res, errors) =
            juniper::execute_sync(r#"query { count }"#, None, &schema, &Variables::new(), &ctx)
                .unwrap();
        // assert
        assert!(res.is_null());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].error().message().contains("oh no"));
    }

    #[test]
    fn test_query_media_types_ok() {
        // arrange
        let expected = vec![
            LabeledCount {
                label: "image/jpeg".to_owned(),
                count: 42,
            },
            LabeledCount {
                label: "video/mpeg".to_owned(),
                count: 101,
            },
            LabeledCount {
                label: "text/plain".to_owned(),
                count: 14,
            },
        ];
        let mut mock = MockEntityDataSource::new();
        mock.expect_all_media_types()
            .returning(move || Ok(expected.clone()));
        let datasource: Arc<dyn EntityDataSource> = Arc::new(mock);
        let assets_path = Box::new(PathBuf::from("/tmp"));
        let ctx = Arc::new(GraphContext::new(datasource, assets_path));
        // act
        let schema = create_schema();
        let (res, _errors) = juniper::execute_sync(
            r#"query { mediaTypes { label count } }"#,
            None,
            &schema,
            &Variables::new(),
            &ctx,
        )
        .unwrap();
        // assert
        let res = res.as_object_value().unwrap();
        let res = res.get_field_value("mediaTypes").unwrap();
        let list_result = res.as_list_value().unwrap();
        let labels = ["image/jpeg", "video/mpeg", "text/plain"];
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
    fn test_query_media_types_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_all_media_types()
            .returning(|| Err(anyhow!("oh no")));
        let datasource: Arc<dyn EntityDataSource> = Arc::new(mock);
        let assets_path = Box::new(PathBuf::from("/tmp"));
        let ctx = Arc::new(GraphContext::new(datasource, assets_path));
        // act
        let schema = create_schema();
        let (res, errors) = juniper::execute_sync(
            r#"query { mediaTypes { label count } }"#,
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
            import_date: make_date_time(2018, 5, 31, 21, 10, 11),
            caption: None,
            location: Some(Location::new("hawaii")),
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let mut mock = MockEntityDataSource::new();
        mock.expect_get_asset_by_digest()
            .with(eq("cafebabe"))
            .returning(move |_| Ok(Some(asset1.clone())));
        let datasource: Arc<dyn EntityDataSource> = Arc::new(mock);
        let assets_path = Box::new(PathBuf::from("/tmp"));
        let ctx = Arc::new(GraphContext::new(datasource, assets_path));
        // act
        let schema = create_schema();
        let query = r#"query Lookup($checksum: String!) {
            lookup(checksum: $checksum) { id }
        }"#;
        let mut vars = Variables::new();
        vars.insert("checksum".to_owned(), InputValue::scalar("cafebabe"));
        let (res, _errors) = juniper::execute_sync(query, None, &schema, &vars, &ctx).unwrap();
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
        mock.expect_get_asset_by_digest()
            .with(eq("cafebabe"))
            .returning(|_| Ok(None));
        let datasource: Arc<dyn EntityDataSource> = Arc::new(mock);
        let assets_path = Box::new(PathBuf::from("/tmp"));
        let ctx = Arc::new(GraphContext::new(datasource, assets_path));
        // act
        let schema = create_schema();
        let query = r#"query Lookup($checksum: String!) {
            lookup(checksum: $checksum) { id }
        }"#;
        let mut vars = Variables::new();
        vars.insert("checksum".to_owned(), InputValue::scalar("cafebabe"));
        let (res, errors) = juniper::execute_sync(query, None, &schema, &vars, &ctx).unwrap();
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
        mock.expect_get_asset_by_digest()
            .with(eq("cafebabe"))
            .returning(|_| Err(anyhow!("oh no")));
        let datasource: Arc<dyn EntityDataSource> = Arc::new(mock);
        let assets_path = Box::new(PathBuf::from("/tmp"));
        let ctx = Arc::new(GraphContext::new(datasource, assets_path));
        // act
        let schema = create_schema();
        let query = r#"query Lookup($checksum: String!) {
            lookup(checksum: $checksum) { id }
        }"#;
        let mut vars = Variables::new();
        vars.insert("checksum".to_owned(), InputValue::scalar("cafebabe"));
        let (res, errors) = juniper::execute_sync(query, None, &schema, &vars, &ctx).unwrap();
        // assert
        let res = res.as_object_value().unwrap();
        let res = res.get_field_value("lookup").unwrap();
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
            .returning(move || Ok(expected.clone()));
        let datasource: Arc<dyn EntityDataSource> = Arc::new(mock);
        let assets_path = Box::new(PathBuf::from("/tmp"));
        let ctx = Arc::new(GraphContext::new(datasource, assets_path));
        // act
        let schema = create_schema();
        let (res, _errors) = juniper::execute_sync(
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
        mock.expect_all_tags().returning(|| Err(anyhow!("oh no")));
        let datasource: Arc<dyn EntityDataSource> = Arc::new(mock);
        let assets_path = Box::new(PathBuf::from("/tmp"));
        let ctx = Arc::new(GraphContext::new(datasource, assets_path));
        // act
        let schema = create_schema();
        let (res, errors) = juniper::execute_sync(
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
            .returning(move || Ok(expected.clone()));
        let datasource: Arc<dyn EntityDataSource> = Arc::new(mock);
        let assets_path = Box::new(PathBuf::from("/tmp"));
        let ctx = Arc::new(GraphContext::new(datasource, assets_path));
        // act
        let schema = create_schema();
        let (res, _errors) = juniper::execute_sync(
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
        mock.expect_all_years().returning(|| Err(anyhow!("oh no")));
        let datasource: Arc<dyn EntityDataSource> = Arc::new(mock);
        let assets_path = Box::new(PathBuf::from("/tmp"));
        let ctx = Arc::new(GraphContext::new(datasource, assets_path));
        // act
        let schema = create_schema();
        let (res, errors) = juniper::execute_sync(
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
