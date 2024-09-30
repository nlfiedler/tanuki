//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::data::repositories::geo::find_location_repository;
use crate::data::repositories::{BlobRepositoryImpl, RecordRepositoryImpl};
use crate::data::sources::EntityDataSource;
use crate::domain::entities::{Asset, LabeledCount, Location, SearchResult};
use crate::domain::usecases::analyze::Counts;
use crate::domain::usecases::diagnose::Diagnosis;
use crate::preso::common::SearchMeta;
use chrono::prelude::*;
use juniper::{
    EmptySubscription, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLScalar, InputValue,
    ParseScalarResult, ParseScalarValue, RootNode, ScalarToken, ScalarValue, Value,
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

#[derive(GraphQLEnum)]
pub enum SortField {
    Date,
    Identifier,
    Filename,
    MediaType,
}

impl From<SortField> for crate::domain::entities::SortField {
    fn from(val: SortField) -> Self {
        match val {
            SortField::Date => crate::domain::entities::SortField::Date,
            SortField::Identifier => crate::domain::entities::SortField::Identifier,
            SortField::Filename => crate::domain::entities::SortField::Filename,
            SortField::MediaType => crate::domain::entities::SortField::MediaType,
        }
    }
}

#[derive(GraphQLEnum)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl From<SortOrder> for crate::domain::entities::SortOrder {
    fn from(val: SortOrder) -> Self {
        match val {
            SortOrder::Ascending => crate::domain::entities::SortOrder::Ascending,
            SortOrder::Descending => crate::domain::entities::SortOrder::Descending,
        }
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
    /// Find assets whose media type (e.g. `image/jpeg`) matches the one given.
    pub media_type: Option<String>,
    /// Field by which to sort the results.
    pub sort_field: Option<SortField>,
    /// Order by which to sort the results.
    pub sort_order: Option<SortOrder>,
}

impl From<SearchParams> for crate::domain::usecases::search::Params {
    fn from(val: SearchParams) -> Self {
        crate::domain::usecases::search::Params {
            tags: val.tags.unwrap_or(vec![]),
            locations: val.locations.unwrap_or(vec![]),
            filename: val.filename,
            media_type: val.media_type,
            before_date: val.before,
            after_date: val.after,
            sort_field: Some(
                val.sort_field
                    .map_or(crate::domain::entities::SortField::Date, |v| v.into()),
            ),
            sort_order: Some(
                val.sort_order
                    .map_or(crate::domain::entities::SortOrder::Descending, |v| v.into()),
            ),
        }
    }
}

#[juniper::graphql_object(
    description = "An attribute name and the number of assets it references."
)]
impl SearchResult {
    /// The identifier of the matching asset.
    fn id(&self) -> String {
        self.asset_id.clone()
    }

    /// The filename for the matching asset.
    fn filename(&self) -> String {
        self.filename.clone()
    }

    /// Media type of the asset.
    fn media_type(&self) -> String {
        self.media_type.clone()
    }

    /// The location for the matching asset, if available.
    fn location(&self) -> Option<Location> {
        self.location.clone()
    }

    /// The date/time for the matching asset.
    fn datetime(&self) -> DateTime<Utc> {
        self.datetime
    }
}

#[juniper::graphql_object(description = "`SearchMeta` is returned from the `search` query.")]
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

/// `EditFilter` is used to select assets when performing a bulk edit.
#[derive(Clone, GraphQLInputObject)]
pub struct EditFilter {
    /// Asset must have all of these tags.
    pub tags: Vec<String>,
    /// Asset location must match defined fields of location. If any field is an
    /// empty string, then the corresponding asset field must be undefined.
    pub location: Option<LocationInput>,
    /// Asset "best" date must be before this date.
    pub before: Option<DateTime<Utc>>,
    /// Asset "best" date must be after this date.
    pub after: Option<DateTime<Utc>>,
    /// Asset media type must match this value.
    pub media_type: Option<String>,
}

impl From<EditFilter> for crate::domain::usecases::edit::Filter {
    fn from(val: EditFilter) -> Self {
        crate::domain::usecases::edit::Filter {
            tags: val.tags,
            location: val.location.map(|l| l.into()),
            before_date: val.before,
            after_date: val.after,
            media_type: val.media_type,
        }
    }
}

#[derive(Clone, GraphQLEnum)]
pub enum TagAction {
    Add,
    Remove,
}

/// `EditTag` is the operation to perform on the asset tags.
#[derive(Clone, GraphQLInputObject)]
pub struct EditTag {
    /// Action to take on the tags list.
    action: TagAction,
    /// Name of the tag to be added or removed.
    value: String,
}

impl From<EditTag> for crate::domain::usecases::edit::TagOperation {
    fn from(val: EditTag) -> Self {
        match val.action {
            TagAction::Add => crate::domain::usecases::edit::TagOperation::Add(val.value),
            TagAction::Remove => crate::domain::usecases::edit::TagOperation::Remove(val.value),
        }
    }
}

#[derive(Clone, GraphQLEnum)]
pub enum LocationAction {
    Set,
    Clear,
}

#[derive(Clone, GraphQLEnum)]
pub enum LocationField {
    Label,
    City,
    Region,
}

impl From<LocationField> for crate::domain::usecases::edit::LocationField {
    fn from(val: LocationField) -> Self {
        match val {
            LocationField::Label => crate::domain::usecases::edit::LocationField::Label,
            LocationField::City => crate::domain::usecases::edit::LocationField::City,
            LocationField::Region => crate::domain::usecases::edit::LocationField::Region,
        }
    }
}

/// `EditLocation` indicates what action to take on the location.
#[derive(Clone, GraphQLInputObject)]
pub struct EditLocation {
    /// Field of the location record to be modified.
    field: LocationField,
    /// Action to take on the location field.
    action: LocationAction,
    /// Value for setting the corresponding location field.
    value: Option<String>,
}

impl From<EditLocation> for crate::domain::usecases::edit::LocationOperation {
    fn from(val: EditLocation) -> Self {
        let field: crate::domain::usecases::edit::LocationField = val.field.into();
        let empty = String::from("oops");
        match val.action {
            LocationAction::Set => crate::domain::usecases::edit::LocationOperation::Set(
                field,
                val.value.unwrap_or(empty),
            ),
            LocationAction::Clear => crate::domain::usecases::edit::LocationOperation::Clear(field),
        }
    }
}

#[derive(Clone, GraphQLEnum)]
pub enum DatetimeAction {
    /// Set the "user" date to the value given.
    Set,
    /// Add the given number of days to the best date, save as "user" date.
    Add,
    /// Subtract the given number of days from the best date, save as "user" date.
    Subtract,
    /// Clear the "user" date field.
    Clear,
}

/// `EditDatetime` indicates what action to take on the asset date/time.
#[derive(Clone, GraphQLInputObject)]
pub struct EditDatetime {
    /// Action to take regarding the asset date/time.
    action: DatetimeAction,
    /// New date/time to apply to the asset.
    value: Option<DateTime<Utc>>,
    /// Number of days to add or remove from the "best" asset date/time.
    delta: Option<i32>,
}

impl From<EditDatetime> for crate::domain::usecases::edit::DatetimeOperation {
    fn from(val: EditDatetime) -> Self {
        let value = val.value.unwrap_or(Utc::now());
        let delta = val.delta.unwrap_or(0) as u16;
        match val.action {
            DatetimeAction::Set => crate::domain::usecases::edit::DatetimeOperation::Set(value),
            DatetimeAction::Add => crate::domain::usecases::edit::DatetimeOperation::Add(delta),
            DatetimeAction::Subtract => {
                crate::domain::usecases::edit::DatetimeOperation::Subtract(delta)
            }
            DatetimeAction::Clear => crate::domain::usecases::edit::DatetimeOperation::Clear,
        }
    }
}

/// `EditParams` specify a filter to select assets and actions to perform on those assets.
#[derive(Clone, GraphQLInputObject)]
pub struct EditParams {
    /// Criteria for finding assets to be modified.
    pub filter: EditFilter,
    /// Operations to perform on the tags.
    pub tags: Vec<EditTag>,
    /// Operations to perform on the location fields.
    pub location: Vec<EditLocation>,
    /// Optional date/time operation to perform.
    pub datetime: Option<EditDatetime>,
}

impl From<EditParams> for crate::domain::usecases::edit::Params {
    fn from(val: EditParams) -> Self {
        crate::domain::usecases::edit::Params {
            filter: val.filter.into(),
            tag_ops: val.tags.into_iter().map(|v| v.into()).collect(),
            location_ops: val.location.into_iter().map(|v| v.into()).collect(),
            datetime_op: val.datetime.map(|v| v.into()),
        }
    }
}

/// `Location` is used to update the location field of an asset.
#[derive(Clone, GraphQLInputObject)]
pub struct LocationInput {
    /// New value for the label of the location.
    label: Option<String>,
    /// New value for the city.
    city: Option<String>,
    /// New value for the region.
    region: Option<String>,
}

impl From<LocationInput> for Location {
    fn from(val: LocationInput) -> Self {
        Location {
            label: val.label,
            city: val.city,
            region: val.region,
        }
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

    /// Retrieve the list of location parts and their associated asset count.
    ///
    /// Parts include the location label split on commas, and the city and
    /// region, if available.
    fn locations(#[graphql(ctx)] ctx: &GraphContext) -> FieldResult<Vec<LabeledCount>> {
        use crate::domain::usecases::location::PartedLocations;
        use crate::domain::usecases::{NoParams, UseCase};
        let repo = RecordRepositoryImpl::new(ctx.datasource.clone());
        let usecase = PartedLocations::new(Box::new(repo));
        let locations: Vec<LabeledCount> = usecase.call(NoParams {})?;
        Ok(locations)
    }

    /// Retrieve the list of unique locations with their full structure.
    fn all_locations(#[graphql(ctx)] ctx: &GraphContext) -> FieldResult<Vec<Location>> {
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
        let usecase = Geocoder::new(Box::new(repo), Box::new(blobs), geocoder);
        let result = usecase.call(Params::new(overwrite));
        if let Ok(count) = result {
            return Ok(count as i32);
        }
        error!("geocode error: {:?}", result);
        Ok(-1)
    }

    /// Perform a search and replace of all of the assets.
    fn edit(#[graphql(ctx)] ctx: &GraphContext, params: EditParams) -> FieldResult<i32> {
        use crate::domain::usecases::edit::{EditAssets, Params};
        use crate::domain::usecases::UseCase;
        let repo = RecordRepositoryImpl::new(ctx.datasource.clone());
        let usecase = EditAssets::new(Box::new(repo));
        let parms: Params = params.into();
        let results: u64 = usecase.call(parms)?;
        Ok(results as i32)
    }

    /// Fill in city and region for locations whose label matches a query.
    fn relocate(
        #[graphql(ctx)] ctx: &GraphContext,
        query: String,
        city: String,
        region: String,
        clear_label: Option<bool>,
    ) -> FieldResult<i32> {
        use crate::domain::usecases::relocate::{Params, Relocate};
        use crate::domain::usecases::UseCase;
        let repo = RecordRepositoryImpl::new(ctx.datasource.clone());
        let usecase = Relocate::new(Box::new(repo));
        let params = Params::new(query, city, region, clear_label.unwrap_or(false));
        let results: u64 = usecase.call(params)?;
        Ok(results as i32)
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
        let usecase = Load::new(Box::new(repo));
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
        mock.expect_get_asset()
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
        mock.expect_get_asset()
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
            .returning(move || Ok(expected.clone()));
        let datasource: Arc<dyn EntityDataSource> = Arc::new(mock);
        let assets_path = Box::new(PathBuf::from("/tmp"));
        let ctx = Arc::new(GraphContext::new(datasource, assets_path));
        // act
        let schema = create_schema();
        let (res, _errors) = juniper::execute_sync(
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
            .returning(|| Err(anyhow!("oh no")));
        let datasource: Arc<dyn EntityDataSource> = Arc::new(mock);
        let assets_path = Box::new(PathBuf::from("/tmp"));
        let ctx = Arc::new(GraphContext::new(datasource, assets_path));
        // act
        let schema = create_schema();
        let (res, errors) = juniper::execute_sync(
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
        mock.expect_query_by_checksum()
            .with(eq("cafebabe"))
            .returning(move |_| Ok(Some("abc123".to_owned())));
        mock.expect_get_asset()
            .with(eq("abc123"))
            .returning(move |_| Ok(asset1.clone()));
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
        mock.expect_query_by_checksum()
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
        mock.expect_query_by_checksum()
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
