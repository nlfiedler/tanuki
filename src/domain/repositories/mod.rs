//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::domain::entities::{Asset, LabeledCount, SearchResult};
use chrono::prelude::*;
use failure::Error;
#[cfg(test)]
use mockall::{automock, predicate::*};
use std::path::{Path, PathBuf};

///
/// Repository for entity records.
///
#[cfg_attr(test, automock)]
pub trait RecordRepository {
    /// Retrieve an asset by its unique identifier.
    fn get_asset(&self, asset_id: &str) -> Result<Asset, Error>;

    /// Attempt to find an asset by SHA-256 hash digest.
    fn get_asset_by_digest(&self, digest: &str) -> Result<Option<Asset>, Error>;

    /// Store the asset entity in the data storage system.
    fn put_asset(&self, asset: &Asset) -> Result<(), Error>;

    /// Retrieve the media type for the identified asset.
    fn get_media_type(&self, asset_id: &str) -> Result<String, Error>;

    /// Return the number of assets stored in the storage system.
    fn count_assets(&self) -> Result<u64, Error>;

    /// Return all of the known locations and the number of assets associated
    /// with each location.
    fn all_locations(&self) -> Result<Vec<LabeledCount>, Error>;

    /// Return all of the known years and the number of assets associated with
    /// each year.
    fn all_years(&self) -> Result<Vec<LabeledCount>, Error>;

    /// Return all of the known tags and the number of assets associated with
    /// each tag.
    fn all_tags(&self) -> Result<Vec<LabeledCount>, Error>;

    /// Search for assets that have all of the given tags.
    fn query_by_tags(&self, tags: Vec<String>) -> Result<Vec<SearchResult>, Error>;

    /// Search for assets that have any of the given locations.
    fn query_by_locations(&self, locations: Vec<String>) -> Result<Vec<SearchResult>, Error>;

    /// Search for assets whose file name matches the one given.
    fn query_by_filename(&self, filename: &str) -> Result<Vec<SearchResult>, Error>;

    /// Search for assets whose media type matches the one given.
    fn query_by_mimetype(&self, mimetype: &str) -> Result<Vec<SearchResult>, Error>;

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

    /// Return the full path to the asset in blob storage.
    fn blob_path(&self, asset_id: &str) -> Result<PathBuf, Error>;

    /// Produce a thumbnail of the desired size for the asset.
    fn thumbnail(&self, width: u32, height: u32, asset_id: &str) -> Result<Vec<u8>, Error>;
}
