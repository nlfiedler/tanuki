//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::data::models::AssetModel;
use crate::domain::entities::{Asset, LabeledCount, SearchResult};
use chrono::prelude::*;
use failure::{err_msg, Error};
#[cfg(test)]
use mockall::automock;
use mokuroku::{base32, Document, Emitter, QueryResult};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::path::Path;
use std::str;

mod database;

/// Data source for entity objects.
#[cfg_attr(test, automock)]
pub trait EntityDataSource {
    /// Retrieve the asset record with the given identifier.
    fn get_asset(&self, asset_id: &str) -> Result<Asset, Error>;

    /// Store the asset record in the database.
    fn put_asset(&self, asset: &Asset) -> Result<(), Error>;

    /// Remove the asset record from the database.
    fn delete_asset(&self, asset_id: &str) -> Result<(), Error>;

    /// Search for the asset with the given hash digest.
    ///
    /// Returns the asset identifier.
    fn query_by_checksum(&self, digest: &str) -> Result<Option<String>, Error>;

    /// Search for assets that have all of the given tags.
    fn query_by_tags(&self, tags: Vec<String>) -> Result<Vec<SearchResult>, Error>;

    /// Search for assets that have any of the given locations.
    fn query_by_locations(&self, locations: Vec<String>) -> Result<Vec<SearchResult>, Error>;

    /// Search for assets whose file name matches the one given.
    fn query_by_filename(&self, filename: &str) -> Result<Vec<SearchResult>, Error>;

    /// Search for assets whose media type matches the one given.
    fn query_by_media_type(&self, mimetype: &str) -> Result<Vec<SearchResult>, Error>;

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

    /// Return the number of assets stored in the data source.
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

    /// Return all of the known tags and the number of assets associated with
    /// each tag.
    fn all_media_types(&self) -> Result<Vec<LabeledCount>, Error>;

    /// Return all asset identifiers in the database.
    fn all_assets(&self) -> Result<Vec<String>, Error>;
}

/// Implementation of the entity data source utilizing mokuroku to manage
/// secondary indices in an instance of RocksDB.
pub struct EntityDataSourceImpl {
    database: database::Database,
}

impl EntityDataSourceImpl {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, Error> {
        // the views provided by our Document implementations
        let views = vec![
            "by_checksum",
            "by_date",
            "by_filename",
            "by_location",
            "by_media_type",
            "by_tags",
            "by_year",
            "newborn",
        ];
        std::fs::create_dir_all(&db_path)?;
        let database = database::Database::new(db_path, views, Box::new(mapper))?;
        Ok(Self { database })
    }

    fn count_all_keys(&self, view: &str) -> Result<Vec<LabeledCount>, Error> {
        let map = self.database.count_all_keys(view)?;
        let results: Vec<LabeledCount> = map
            .iter()
            .map(|t| LabeledCount {
                label: String::from_utf8((*t.0).to_vec()).unwrap(),
                count: *t.1,
            })
            .collect();
        Ok(results)
    }
}

impl EntityDataSource for EntityDataSourceImpl {
    fn get_asset(&self, asset_id: &str) -> Result<Asset, Error> {
        let db_key = format!("asset/{}", asset_id);
        let maybe_asset = self.database.get_asset(&db_key)?;
        maybe_asset.ok_or_else(|| err_msg(format!("missing asset {}", asset_id)))
    }

    fn put_asset(&self, asset: &Asset) -> Result<(), Error> {
        let key = format!("asset/{}", asset.key);
        self.database.put_asset(&key, asset)?;
        Ok(())
    }

    fn delete_asset(&self, asset_id: &str) -> Result<(), Error> {
        let key = format!("asset/{}", asset_id);
        self.database.delete_document(&key.as_bytes())?;
        Ok(())
    }

    fn query_by_checksum(&self, digest: &str) -> Result<Option<String>, Error> {
        // secondary index keys are lowercase
        let digest = digest.to_lowercase();
        let maybe_value = self.database.query_one_by_key("by_checksum", digest)?;
        Ok(maybe_value.map(|v| {
            let vec: Vec<u8> = Vec::from(v.doc_id.as_ref());
            // Remove the leading 'asset/' on the document identifier for
            // those records that contribute to this particular index.
            String::from_utf8(vec).unwrap().split_off(6)
        }))
    }

    fn query_by_tags(&self, tags: Vec<String>) -> Result<Vec<SearchResult>, Error> {
        // secondary index keys are lowercase
        let tags: Vec<String> = tags.into_iter().map(|v| v.to_lowercase()).collect();
        let query_results = self.database.query_all_keys("by_tags", tags)?;
        let search_results = convert_results(query_results);
        Ok(search_results)
    }

    fn query_by_locations(&self, locations: Vec<String>) -> Result<Vec<SearchResult>, Error> {
        // secondary index keys are lowercase
        let locations: Vec<String> = locations.into_iter().map(|v| v.to_lowercase()).collect();
        // query each of the keys and collect the results into one list
        let mut query_results: Vec<QueryResult> = Vec::new();
        for key in locations.iter() {
            let mut results = self.database.query_by_key("by_location", key)?;
            query_results.append(&mut results);
        }
        let search_results = convert_results(query_results);
        Ok(search_results)
    }

    fn query_by_filename(&self, filename: &str) -> Result<Vec<SearchResult>, Error> {
        // secondary index keys are lowercase
        let filename = filename.to_lowercase();
        let query_results = self.database.query_by_key("by_filename", filename)?;
        let search_results = convert_results(query_results);
        Ok(search_results)
    }

    fn query_by_media_type(&self, mimetype: &str) -> Result<Vec<SearchResult>, Error> {
        // secondary index keys are lowercase
        let mimetype = mimetype.to_lowercase();
        let query_results = self.database.query_by_key("by_media_type", mimetype)?;
        let search_results = convert_results(query_results);
        Ok(search_results)
    }

    fn query_before_date(&self, before: DateTime<Utc>) -> Result<Vec<SearchResult>, Error> {
        let key = encode_datetime(&before);
        let query_results = self.database.query_less_than("by_date", key)?;
        let search_results = convert_results(query_results);
        Ok(search_results)
    }

    fn query_after_date(&self, after: DateTime<Utc>) -> Result<Vec<SearchResult>, Error> {
        let key = encode_datetime(&after);
        let query_results = self.database.query_greater_than("by_date", key)?;
        let search_results = convert_results(query_results);
        Ok(search_results)
    }

    fn query_date_range(
        &self,
        after: DateTime<Utc>,
        before: DateTime<Utc>,
    ) -> Result<Vec<SearchResult>, Error> {
        let key_a = encode_datetime(&after);
        let key_b = encode_datetime(&before);
        let query_results = self.database.query_range("by_date", key_a, key_b)?;
        let search_results = convert_results(query_results);
        Ok(search_results)
    }

    fn query_newborn(&self, after: DateTime<Utc>) -> Result<Vec<SearchResult>, Error> {
        let epoch = Utc.timestamp(0, 0);
        let key = encode_datetime(&after);
        let query_results = if epoch > after {
            // A Unix timestamp is encoded as seconds since the epoch using a
            // two's complement signed integer. Times before the epoch are
            // larger than times after when encoded with base32hex. The encoded
            // values for negative numbers get larger as they approach zero.
            // Thus, to find dates after the pre-epoch date given, first query
            // for anything larger than that value, then query for anything
            // between the zero time and now.
            let mut results1 = self.database.query_greater_than("newborn", key)?;
            let epoch_key = encode_datetime(&epoch);
            let now = Utc::now();
            let now_key = encode_datetime(&now);
            let mut results2 = self.database.query_range("newborn", epoch_key, now_key)?;
            results1.append(&mut results2);
            results1
        } else {
            self.database.query_greater_than("newborn", key)?
        };
        let search_results = convert_results(query_results);
        Ok(search_results)
    }

    fn count_assets(&self) -> Result<u64, Error> {
        let count: usize = self.database.count_prefix("asset/")?;
        Ok(count as u64)
    }

    fn all_locations(&self) -> Result<Vec<LabeledCount>, Error> {
        self.count_all_keys("by_location")
    }

    fn all_years(&self) -> Result<Vec<LabeledCount>, Error> {
        self.count_all_keys("by_year")
    }

    fn all_tags(&self) -> Result<Vec<LabeledCount>, Error> {
        self.count_all_keys("by_tags")
    }

    fn all_media_types(&self) -> Result<Vec<LabeledCount>, Error> {
        self.count_all_keys("by_media_type")
    }

    fn all_assets(&self) -> Result<Vec<String>, Error> {
        self.database.find_prefix("asset/")
    }
}

/// Convert the database query results to our search results.
///
/// Silently drops any results that fail to deserialize.
fn convert_results(query_results: Vec<QueryResult>) -> Vec<SearchResult> {
    let search_results: Vec<SearchResult> = query_results
        .into_iter()
        .filter_map(|r| {
            // ignore any query results that fail to serialize, there is not
            // much that can be done with that now
            SearchResult::try_from(r).ok()
        })
        .collect();
    search_results
}

/// Encode the date/time as a BigEndian base32hex encoded value.
fn encode_datetime(date: &DateTime<Utc>) -> Vec<u8> {
    let millis = date.timestamp_millis();
    let bytes = millis.to_be_bytes().to_vec();
    base32::encode(&bytes)
}

impl Document for Asset {
    fn from_bytes(key: &[u8], value: &[u8]) -> Result<Self, Error> {
        let mut de = serde_cbor::Deserializer::from_slice(value);
        let mut result = AssetModel::deserialize(&mut de)?;
        // remove the "asset/" key prefix added by the data source
        result.key = str::from_utf8(&key[6..])?.to_owned();
        Ok(result)
    }

    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut encoded: Vec<u8> = Vec::new();
        let mut ser = serde_cbor::Serializer::new(&mut encoded);
        AssetModel::serialize(self, &mut ser)?;
        Ok(encoded)
    }

    fn map(&self, view: &str, emitter: &Emitter) -> Result<(), Error> {
        // make the index value assuming we will emit something
        let value = SearchResult::new(self);
        let idv: Vec<u8> = value.to_bytes()?;
        if view == "by_tags" {
            for tag in &self.tags {
                // secondary index keys are lowercase
                let lower = tag.to_lowercase();
                emitter.emit(lower.as_bytes(), Some(&idv))?;
            }
        } else if view == "by_checksum" {
            // secondary index keys are lowercase
            let lower = self.checksum.to_lowercase();
            emitter.emit(lower.as_bytes(), None)?;
        } else if view == "by_filename" {
            // secondary index keys are lowercase
            let lower = self.filename.to_lowercase();
            emitter.emit(lower.as_bytes(), Some(&idv))?;
        } else if view == "by_media_type" {
            // secondary index keys are lowercase
            let lower = self.media_type.to_lowercase();
            emitter.emit(lower.as_bytes(), Some(&idv))?;
        } else if view == "by_location" {
            if let Some(loc) = self.location.as_ref() {
                // secondary index keys are lowercase
                let lower = loc.to_lowercase();
                emitter.emit(lower.as_bytes(), Some(&idv))?;
            }
        } else if view == "by_year" {
            let year = if let Some(ud) = self.user_date.as_ref() {
                ud.year()
            } else if let Some(od) = self.original_date.as_ref() {
                od.year()
            } else {
                self.import_date.year()
            };
            let formatted = format!("{}", year);
            let bytes = formatted.as_bytes();
            emitter.emit(bytes, Some(&idv))?;
        } else if view == "by_date" {
            let best_date = if let Some(ud) = self.user_date.as_ref() {
                encode_datetime(ud)
            } else if let Some(od) = self.original_date.as_ref() {
                encode_datetime(od)
            } else {
                encode_datetime(&self.import_date)
            };
            emitter.emit(&best_date, Some(&idv))?;
        } else if view == "newborn" {
            if self.tags.is_empty() && self.caption.is_none() && self.location.is_none() {
                // use the import date for newborn, not the "best" date
                let import_date = encode_datetime(&self.import_date);
                emitter.emit(&import_date, Some(&idv))?;
            }
        }
        Ok(())
    }
}

pub fn mapper(key: &[u8], value: &[u8], view: &str, emitter: &Emitter) -> Result<(), Error> {
    if &key[..6] == b"asset/".as_ref() {
        let doc = Asset::from_bytes(key, value)?;
        doc.map(view, emitter)?;
    }
    Ok(())
}

/// Database index value for an asset.
#[derive(Serialize, Deserialize)]
#[serde(remote = "SearchResult")]
struct IndexValue {
    #[serde(skip)]
    pub asset_id: String,
    /// Original filename of the asset.
    #[serde(rename = "n")]
    pub filename: String,
    /// Detected media type of the file.
    #[serde(rename = "m")]
    pub media_type: String,
    /// User-defined location of the asset.
    #[serde(rename = "l")]
    pub location: Option<String>,
    /// Best date/time for the indexed asset.
    #[serde(rename = "d")]
    pub datetime: DateTime<Utc>,
}

impl SearchResult {
    // Deserialize from a slice of bytes.
    fn from_bytes(value: &[u8]) -> Result<Self, Error> {
        let mut de = serde_cbor::Deserializer::from_slice(value);
        let result = IndexValue::deserialize(&mut de)?;
        Ok(result)
    }

    // Serialize to a vector of bytes.
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut encoded: Vec<u8> = Vec::new();
        let mut ser = serde_cbor::Serializer::new(&mut encoded);
        IndexValue::serialize(self, &mut ser)?;
        Ok(encoded)
    }
}

impl TryFrom<QueryResult> for SearchResult {
    type Error = failure::Error;

    fn try_from(value: QueryResult) -> Result<Self, Self::Error> {
        let mut result = SearchResult::from_bytes(value.value.as_ref())?;
        // remove the "asset/" key prefix added by the data source
        let key_vec = (*value.doc_id).to_vec();
        result.asset_id = String::from_utf8(key_vec)?.split_off(6);
        Ok(result)
    }
}
