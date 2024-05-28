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
    /// Retrieve an asset by its unique identifier.
    fn get_asset(&self, asset_id: &str) -> Result<Asset, Error>;

    /// Attempt to find an asset by SHA-256 hash digest.
    fn get_asset_by_digest(&self, digest: &str) -> Result<Option<Asset>, Error>;

    /// Store the asset entity in the data storage system.
    fn put_asset(&self, asset: &Asset) -> Result<(), Error>;

    /// Remove the asset record from the database.
    fn delete_asset(&self, asset_id: &str) -> Result<(), Error>;

    /// Return the number of assets stored in the storage system.
    fn count_assets(&self) -> Result<u64, Error>;

    /// Return all of the known locations and the number of assets associated
    /// with each location. Results include those processed by splitting on commas.
    fn all_locations(&self) -> Result<Vec<LabeledCount>, Error>;

    /// Return all of the unique locations with all available field values.
    fn raw_locations(&self) -> Result<Vec<Location>, Error>;

    /// Return all of the known years and the number of assets associated with
    /// each year.
    fn all_years(&self) -> Result<Vec<LabeledCount>, Error>;

    /// Return all of the known tags and the number of assets associated with
    /// each tag.
    fn all_tags(&self) -> Result<Vec<LabeledCount>, Error>;

    /// Return all of the known media types and the number of assets associated
    /// with each type.
    fn all_media_types(&self) -> Result<Vec<LabeledCount>, Error>;

    /// Return all asset identifiers in the database.
    fn all_assets(&self) -> Result<Vec<String>, Error>;

    /// Search for assets that have all of the given tags.
    fn query_by_tags(&self, tags: Vec<String>) -> Result<Vec<SearchResult>, Error>;

    /// Search for assets that have any of the given locations.
    fn query_by_locations(&self, locations: Vec<String>) -> Result<Vec<SearchResult>, Error>;

    /// Search for assets whose file name matches the one given.
    fn query_by_filename(&self, filename: &str) -> Result<Vec<SearchResult>, Error>;

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
