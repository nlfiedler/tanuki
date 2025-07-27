//
// Copyright (c) 2025 Nathan Fiedler
//
use crate::domain::entities::{Asset, LabeledCount, Location, SearchResult};
use crate::domain::repositories::FetchedAssets;
use anyhow::Error;
use chrono::prelude::*;
#[cfg(test)]
use mockall::automock;
use std::path::Path;
use std::str;
use std::sync::Arc;

pub mod duckdb;
pub mod rocksdb;
pub mod sqlite;

/// Data source for entity records.
#[cfg_attr(test, automock)]
pub trait EntityDataSource: Send + Sync {
    /// Retrieve an asset by its unique identifier, failing if not found.
    fn get_asset_by_id(&self, asset_id: &str) -> Result<Asset, Error>;

    /// Find an asset by its SHA-256 checksum, returning `None` if not found.
    fn get_asset_by_digest(&self, digest: &str) -> Result<Option<Asset>, Error>;

    /// Store the asset entity in the data source either as a new record or
    /// updating an existing record, according to its unique identifier.
    fn put_asset(&self, asset: &Asset) -> Result<(), Error>;

    /// Remove the asset record from the data source.
    fn delete_asset(&self, asset_id: &str) -> Result<(), Error>;

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
    /// As with `query_after_date()`, the after value is inclusive.
    fn query_date_range(
        &self,
        after: DateTime<Utc>,
        before: DateTime<Utc>,
    ) -> Result<Vec<SearchResult>, Error>;

    /// Query for assets that lack any tags, caption, and location.
    fn query_newborn(&self, after: DateTime<Utc>) -> Result<Vec<SearchResult>, Error>;

    /// Return the number of assets stored in the data source.
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

    /// Return all of the known tags and the number of assets associated with
    /// each tag.
    fn all_media_types(&self) -> Result<Vec<LabeledCount>, Error>;

    /// Return all asset identifiers in the data source.
    fn all_assets(&self) -> Result<Vec<String>, Error>;

    /// Return all assets from the data source in no specific order.
    ///
    /// The `cursor` should start as `None` to begin the retrieval from the
    /// "first" assets in the data source. On the next call, the `cursor` should
    /// be the value returned in the `FetchedAssets` in order to continue the
    /// scan through the data source.
    fn fetch_assets(&self, cursor: Option<String>, count: usize) -> Result<FetchedAssets, Error>;

    /// Store multiple assets into the data source.
    ///
    /// Data sources are free to implement this in whatever manner helps with
    /// performance and/or data integrity. For some, this may mean nothing more
    /// than simply iterating over the set and invoking `put_asset()`.
    fn store_assets(&self, incoming: Vec<Asset>) -> Result<(), Error>;
}

///
/// Construct a new entity data source implementation for the given path.
///
/// The particular implementation is chosen using the `DATABASE_TYPE`
/// environment variable, with "sqlite" and "rocksdb" being supported values. If
/// not set, or the value is not recognized, defaults to RocksDB
///
pub fn new_datasource_for_path<P: AsRef<Path>>(
    db_path: P,
) -> Result<Arc<dyn EntityDataSource>, Error> {
    let db_type = std::env::var("DATABASE_TYPE")
        .map(|v| v.to_lowercase())
        .unwrap_or("rocksdb".into());
    let source: Arc<dyn EntityDataSource> = if db_type == "duckdb" {
        Arc::new(duckdb::EntityDataSourceImpl::new(db_path)?)
    } else if db_type == "sqlite" {
        Arc::new(sqlite::EntityDataSourceImpl::new(db_path)?)
    } else {
        Arc::new(rocksdb::EntityDataSourceImpl::new(db_path)?)
    };
    Ok(source)
}
