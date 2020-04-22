//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::domain::entities::{Asset, LabeledCount};
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
    fn blob_path(&self, asset: &Asset) -> Result<PathBuf, Error>;
}
