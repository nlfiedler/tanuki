//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::domain::entities::Asset;
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
    #[allow(clippy::ptr_arg)]
    fn get_asset(&self, asset_id: String) -> Result<Asset, Error>;

    /// Attempt to find an asset by SHA-256 hash digest.
    #[allow(clippy::ptr_arg)]
    fn get_asset_by_digest(&self, digest: String) -> Result<Option<Asset>, Error>;

    /// Store the asset entity in the data source.
    fn put_asset(&self, asset: &Asset) -> Result<(), Error>;

    /// Retrieve the media type for the identified asset.
    #[allow(clippy::ptr_arg)]
    fn get_media_type(&self, asset_id: String) -> Result<String, Error>;
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
