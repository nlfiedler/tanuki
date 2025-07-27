//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::{
    Asset, GeocodedLocation, GlobalPosition, LabeledCount, Location, SearchResult,
};
use anyhow::Error;
use chrono::prelude::*;
#[cfg(test)]
use mockall::{automock, predicate::*};
use std::path::{Path, PathBuf};

///
/// Repository for entity records.
///
#[cfg_attr(test, automock)]
pub trait RecordRepository: Send {
    /// Retrieve an asset by its unique identifier, failing if not found.
    fn get_asset_by_id(&self, asset_id: &str) -> Result<Asset, Error>;

    /// Find an asset by its SHA-256 checksum, returning `None` if not found.
    fn get_asset_by_digest(&self, digest: &str) -> Result<Option<Asset>, Error>;

    /// Store the asset entity in the database either as a new record or
    /// updating an existing record, according to its unique identifier.
    fn put_asset(&self, asset: &Asset) -> Result<(), Error>;

    /// Remove the asset record from the database.
    fn delete_asset(&self, asset_id: &str) -> Result<(), Error>;

    /// Return the number of asset records stored in the database.
    fn count_assets(&self) -> Result<u64, Error>;

    /// Return all of the location values and the number of assets associated
    /// with each value. Values are extracted from each of the parts of the
    /// location field.
    fn all_locations(&self) -> Result<Vec<LabeledCount>, Error>;

    /// Return all of the unique locations with all available field values.
    fn raw_locations(&self) -> Result<Vec<Location>, Error>;

    /// Return all of the years for which their are assests and the number of
    /// assets associated with each year.
    fn all_years(&self) -> Result<Vec<LabeledCount>, Error>;

    /// Return all of the known tags and the number of assets associated with
    /// each tag.
    fn all_tags(&self) -> Result<Vec<LabeledCount>, Error>;

    /// Return all of the known media types and the number of assets associated
    /// with each type.
    fn all_media_types(&self) -> Result<Vec<LabeledCount>, Error>;

    /// Return all asset identifiers in the database.
    fn all_assets(&self) -> Result<Vec<String>, Error>;

    /// Return all assets from the database in no specific order.
    ///
    /// The `cursor` should start as `None` to begin the retrieval from the
    /// "first" assets in the database. On the next call, the `cursor` should be
    /// the value returned in the `FetchedAssets` in order to continue the scan
    /// through the database.
    fn fetch_assets(&self, cursor: Option<String>, count: usize) -> Result<FetchedAssets, Error>;

    /// Search for assets that have all of the given tags.
    fn query_by_tags(&self, tags: Vec<String>) -> Result<Vec<SearchResult>, Error>;

    /// Search for assets whose location fields match all of the given values.
    ///
    /// For example, searching for `["paris","france"]` will return assets that
    /// have both `"paris"` and `"france"` in the location column, such as in
    /// the `city` and `region` fields.
    fn query_by_locations(&self, locations: Vec<String>) -> Result<Vec<SearchResult>, Error>;

    /// Search for assets whose media type matches the one given.
    fn query_by_media_type(&self, media_type: &str) -> Result<Vec<SearchResult>, Error>;

    /// Search for asssets whose best date is before the one given.
    fn query_before_date(&self, before: DateTime<Utc>) -> Result<Vec<SearchResult>, Error>;

    /// Search for asssets whose best date is equal to or after the one given.
    fn query_after_date(&self, after: DateTime<Utc>) -> Result<Vec<SearchResult>, Error>;

    /// Search for assets whose best date is between the after and before dates.
    ///
    /// As with `query_after_date()`, the after value inclusive.
    fn query_date_range(
        &self,
        after: DateTime<Utc>,
        before: DateTime<Utc>,
    ) -> Result<Vec<SearchResult>, Error>;

    /// Query for assets that lack any tags, caption, and location.
    fn query_newborn(&self, after: DateTime<Utc>) -> Result<Vec<SearchResult>, Error>;

    /// Export all of the asset records as JSON to the named file.
    fn dump(&self, filepath: &Path) -> Result<u64, Error>;

    /// Import all of the assets found in the named JSON file.
    fn load(&self, filepath: &Path) -> Result<u64, Error>;
}

///
/// Value returned by `fetch_assets()` that includes the assets retrieved and a
/// cursor-like value that can be used to continue retrieving assets from the
/// last-visited location in the data source. When the returned `cursor` value
/// is `None` then there are no more assets left to be visited.
///
pub struct FetchedAssets {
    pub assets: Vec<Asset>,
    /// If `None`, then there are no more assets available.
    pub cursor: Option<String>,
}

///
/// Repository for asset blobs.
///
#[cfg_attr(test, automock)]
pub trait BlobRepository {
    /// Move the given file into the blob store.
    ///
    /// Existing blobs will not be overwritten.
    fn store_blob(&self, filepath: &Path, asset: &Asset) -> Result<(), Error>;

    /// Move the given file into the blob store, replacing whatever is already
    /// there. Used when an asset is to be replaced by a different version.
    fn replace_blob(&self, filepath: &Path, asset: &Asset) -> Result<(), Error>;

    /// Return the full path to the asset in blob storage.
    fn blob_path(&self, asset_id: &str) -> Result<PathBuf, Error>;

    /// Change the identity of the asset in blob storage.
    fn rename_blob(&self, old_id: &str, new_id: &str) -> Result<(), Error>;

    /// Produce a thumbnail of the desired size for the asset.
    fn thumbnail(&self, width: u32, height: u32, asset_id: &str) -> Result<Vec<u8>, Error>;

    /// Clear the thumbnail cache of any entries for the given asset.
    fn clear_cache(&self, asset_id: &str) -> Result<(), Error>;
}

///
/// Repository for finding a location given a set of GPS coordinates.
///
#[cfg_attr(test, automock)]
pub trait LocationRepository: Send + Sync {
    /// Find a descriptive location for the given global coordinates.
    fn find_location(&self, coords: &GlobalPosition) -> Result<GeocodedLocation, Error>;
}

///
/// Repository for maintaining a cache of search results.
///
#[cfg_attr(test, automock)]
pub trait SearchRepository: Send + Sync {
    /// Save the search results keyed by a query string.
    fn put(&self, key: String, val: Vec<SearchResult>) -> Result<(), Error>;

    /// Find the cached search results for the given query, if any.
    fn get(&self, key: &str) -> Result<Option<Vec<SearchResult>>, Error>;

    /// Clear all cached search results.
    fn clear(&self) -> Result<(), Error>;
}
