//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::data::models::AssetModel;
use crate::domain::entities::Asset;
use chrono::prelude::*;
use failure::{err_msg, Error};
#[cfg(test)]
use mockall::automock;
use mokuroku::{base32, Document, Emitter};
use serde::{Deserialize, Serialize};
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

    /// Search for the asset with the given hash digest.
    ///
    /// Returns the asset identifier.
    fn query_by_checksum(&self, digest: &str) -> Result<Option<String>, Error>;
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
            "by_tag",
        ];
        let database = database::Database::new(db_path, views, Box::new(mapper))?;
        Ok(Self { database })
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

    fn query_by_checksum(&self, _digest: &str) -> Result<Option<String>, Error> {
        Err(err_msg("not yet implemented"))
    }
}

impl Document for Asset {
    fn from_bytes(key: &[u8], value: &[u8]) -> Result<Self, Error> {
        let mut de = serde_cbor::Deserializer::from_slice(value);
        let mut result = AssetModel::deserialize(&mut de)?;
        // remove the "asset/" key prefix added by the database source
        result.key = str::from_utf8(&key[6..])?.to_owned();
        Ok(result)
    }

    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut encoded: Vec<u8> = Vec::new();
        let mut ser = serde_cbor::Serializer::new(&mut encoded);
        AssetModel::serialize(&self, &mut ser)?;
        Ok(encoded)
    }

    fn map(&self, view: &str, emitter: &Emitter) -> Result<(), Error> {
        // make the index value assuming we will emit something
        let value = IndexValue::new(self);
        let idv: Vec<u8> = serde_cbor::to_vec(&value)?;
        if view == "by_tag" {
            for tag in &self.tags {
                emitter.emit(tag.as_bytes(), Some(&idv))?;
            }
        } else if view == "by_checksum" {
            emitter.emit(self.checksum.as_bytes(), None)?;
        } else if view == "by_filename" {
            let lower = self.filename.to_lowercase();
            emitter.emit(lower.as_bytes(), Some(&idv))?;
        } else if view == "by_media_type" {
            let lower = self.media_type.to_lowercase();
            emitter.emit(lower.as_bytes(), Some(&idv))?;
        } else if view == "by_location" {
            if let Some(loc) = self.location.as_ref() {
                let lower = loc.to_lowercase();
                emitter.emit(lower.as_bytes(), Some(&idv))?;
            }
        } else if view == "by_date" {
            let millis = if let Some(ud) = self.user_date.as_ref() {
                ud.timestamp_millis()
            } else if let Some(od) = self.original_date.as_ref() {
                od.timestamp_millis()
            } else {
                self.import_date.timestamp_millis()
            };
            let bytes = millis.to_be_bytes().to_vec();
            let encoded = base32::encode(&bytes);
            emitter.emit(&encoded, Some(&idv))?;
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
struct IndexValue {
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

impl IndexValue {
    /// Build an index value from the given asset.
    pub fn new(asset: &Asset) -> Self {
        let date = if let Some(ud) = asset.user_date.as_ref() {
            ud.to_owned()
        } else if let Some(od) = asset.original_date.as_ref() {
            od.to_owned()
        } else {
            asset.import_date
        };
        Self {
            filename: asset.filename.clone(),
            media_type: asset.media_type.clone(),
            location: asset.location.clone(),
            datetime: date,
        }
    }

    // /// Deserialize from a slice of bytes.
    // pub fn from_bytes(value: &[u8]) -> Result<Self, Error> {
    //     let serde_result: IndexValue = serde_cbor::from_slice(value)?;
    //     Ok(serde_result)
    // }
}
