//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::data::models::AssetModel;
use crate::data::sources::EntityDataSource;
use crate::domain::entities::{Asset, LabeledCount, Location, SearchResult};
use crate::domain::repositories::FetchedAssets;
use anyhow::{anyhow, Error};
use chrono::prelude::*;
use mokuroku::{base32, Document, Emitter, QueryResult};
use rocksdb::{Direction, IteratorMode, Options};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};
use std::str;
use std::sync::{Arc, LazyLock, Mutex};

// Keep a map of references to shared DB instances mapped by the path to the
// database files. RocksDB itself is thread-safe for get/put/write, and the DB
// type implements Send and Sync.
static DBASE_REFS: LazyLock<Mutex<HashMap<PathBuf, Arc<Mutex<mokuroku::Database>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Test only function for dropping the database reference for the given path,
/// which allows for the DBPath test helper to invoke DB::destroy().
pub fn drop_database_ref<P: AsRef<Path>>(db_path: P) {
    let mut db_refs = DBASE_REFS.lock().unwrap();
    db_refs.remove(db_path.as_ref());
}

///
/// An instance of the database for reading and writing records to disk.
///
pub struct Database {
    /// RocksDB instance wrapped with mokuroku.
    db: Arc<Mutex<mokuroku::Database>>,
}

impl Database {
    ///
    /// Create an instance of Database using the given path for storage. Will
    /// reuse an existing `DB` instance for the given path, if one has already
    /// been opened.
    ///
    pub fn new<P: AsRef<Path>, I, N>(
        db_path: P,
        views: I,
        mapper: mokuroku::ByteMapper,
    ) -> Result<Self, Error>
    where
        I: IntoIterator<Item = N>,
        N: ToString,
    {
        // should be able to recover from a poisoned mutex without any problem
        let mut db_refs = DBASE_REFS.lock().unwrap();
        if let Some(arc) = db_refs.get(db_path.as_ref()) {
            return Ok(Self { db: arc.clone() });
        }
        let buf = db_path.as_ref().to_path_buf();
        // prevent the proliferation of old log files
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_keep_log_file_num(10);
        // Set the max open files; the default (-1) keeps all of the files open,
        // which is simply insane for desktop systems like macOS, where the
        // default max open files (ulimit -a) is 256.
        opts.set_max_open_files(128);
        let db = mokuroku::Database::open(db_path.as_ref(), views, mapper, opts)?;
        let arc = Arc::new(Mutex::new(db));
        db_refs.insert(buf, arc.clone());
        Ok(Self { db: arc })
    }

    ///
    /// Delete the database record associated with the given key.
    ///
    pub fn delete_document(&self, key: &[u8]) -> Result<(), Error> {
        let mut db = self.db.lock().unwrap();
        db.delete(key)?;
        Ok(())
    }

    ///
    /// Put the given asset into the database.
    ///
    /// The `Asset` will be serialized via the `Document` interface.
    ///
    pub fn put_asset(&self, key: &str, asset: &Asset) -> Result<(), Error> {
        let mut db = self.db.lock().unwrap();
        db.put(key, asset)?;
        Ok(())
    }

    ///
    /// Retrieve the asset by the given key, returning None if not found.
    ///
    /// The `Asset` will be deserialized via the `Document` interface.
    ///
    pub fn get_asset(&self, key: &str) -> Result<Option<Asset>, Error> {
        let db = self.db.lock().unwrap();
        Ok(db.get(key)?)
    }

    ///
    /// Query the named view by a certain key, returning at most one item.
    ///
    pub fn query_one_by_key<K: AsRef<[u8]>>(
        &self,
        view: &str,
        key: K,
    ) -> Result<Option<mokuroku::QueryResult>, Error> {
        let mut db = self.db.lock().unwrap();
        let mut iter = db.query_by_key(view, key)?;
        Ok(iter.next())
    }

    ///
    /// Query the named view by a certain key.
    ///
    pub fn query_by_key<K: AsRef<[u8]>>(
        &self,
        view: &str,
        key: K,
    ) -> Result<Vec<mokuroku::QueryResult>, Error> {
        let mut db = self.db.lock().unwrap();
        // add the index key separator to get an "exact" match
        let mut exact_key: Vec<u8> = Vec::from(key.as_ref());
        exact_key.push(0);
        let iter = db.query_by_key(view, &exact_key)?;
        Ok(iter.collect())
    }

    ///
    /// Query the index for documents that have all of the given keys.
    ///
    pub fn query_all_keys<I, N>(
        &self,
        view: &str,
        keys: I,
    ) -> Result<Vec<mokuroku::QueryResult>, Error>
    where
        I: IntoIterator<Item = N>,
        N: AsRef<[u8]>,
    {
        let mut db = self.db.lock().unwrap();
        let exact_keys: Vec<Vec<u8>> = keys
            .into_iter()
            .map(|v| {
                let mut exact_key = Vec::from(v.as_ref());
                exact_key.push(0);
                exact_key
            })
            .collect();
        Ok(db.query_all_keys(view, exact_keys)?)
    }

    ///
    /// Query an index by range of key values, returning those results whose key
    /// is equal to or greater than the first key, and less than the second key.
    ///
    pub fn query_range<K: AsRef<[u8]>>(
        &self,
        view: &str,
        key_a: K,
        key_b: K,
    ) -> Result<Vec<mokuroku::QueryResult>, Error> {
        let mut db = self.db.lock().unwrap();
        let iter = db.query_range(view, key_a, key_b)?;
        Ok(iter.collect())
    }

    ///
    /// Query on the given index, returning those results whose key is *equal*
    /// to or *greater than* the key.
    ///
    pub fn query_greater_than<K: AsRef<[u8]>>(
        &self,
        view: &str,
        key: K,
    ) -> Result<Vec<mokuroku::QueryResult>, Error> {
        let mut db = self.db.lock().unwrap();
        let iter = db.query_greater_than(view, key)?;
        Ok(iter.collect())
    }

    ///
    /// Query on the given index, returning those results whose key strictly
    /// *less than* the key.
    ///
    pub fn query_less_than<K: AsRef<[u8]>>(
        &self,
        view: &str,
        key: K,
    ) -> Result<Vec<mokuroku::QueryResult>, Error> {
        let mut db = self.db.lock().unwrap();
        let iter = db.query_less_than(view, key)?;
        Ok(iter.collect())
    }

    ///
    /// Count those keys that start with the given prefix.
    ///
    pub fn count_prefix(&self, prefix: &str) -> Result<usize, Error> {
        let pre_bytes = prefix.as_bytes();
        // this only gets us started, we then have to check for the end of the range
        let db = self.db.lock().unwrap();
        let iter = db.db().prefix_iterator(pre_bytes);
        let mut count = 0;
        for item in iter {
            let (key, _value) = item?;
            let pre = &key[..pre_bytes.len()];
            if pre != pre_bytes {
                break;
            }
            count += 1;
        }
        Ok(count)
    }

    ///
    /// Query the index and return the number of occurrences of all keys.
    ///
    /// The map keys are the index keys, and the map values are the number of
    /// times each key was encountered in the index.
    ///
    pub fn count_all_keys(&self, view: &str) -> Result<HashMap<Box<[u8]>, usize>, Error> {
        let mut db = self.db.lock().unwrap();
        Ok(db.count_all_keys(view)?)
    }

    ///
    /// Find all those keys that start with the given prefix.
    ///
    /// Returns the key without the prefix.
    ///
    pub fn find_prefix(&self, prefix: &str) -> Result<Vec<String>, Error> {
        let pre_bytes = prefix.as_bytes();
        // this only gets us started, we then have to check for the end of the range
        let db = self.db.lock().unwrap();
        let iter = db.db().prefix_iterator(pre_bytes);
        let mut results: Vec<String> = Vec::new();
        for item in iter {
            let (key, _value) = item?;
            let pre = &key[..pre_bytes.len()];
            if pre != pre_bytes {
                break;
            }
            let key_str = str::from_utf8(&key[pre_bytes.len()..])?;
            results.push(key_str.to_owned());
        }
        Ok(results)
    }

    ///
    /// Find all keys and values such that the key starts with the given prefix,
    /// optionally seeking to the key given in `cursor`, and returning up to
    /// `count` items.
    ///
    /// Unlike other queries, this function returns the raw key/value pairs.
    ///
    pub fn scan(
        &self,
        prefix: &str,
        cursor: Option<String>,
        count: usize,
    ) -> Result<Vec<(Box<[u8]>, Box<[u8]>)>, Error> {
        let prefix_bytes = prefix.as_bytes();
        let db = self.db.lock().unwrap();
        let mut iter = db.db().prefix_iterator(prefix_bytes);
        if let Some(from_str) = cursor {
            let from_bytes = from_str.as_bytes();
            iter.set_mode(IteratorMode::From(from_bytes, Direction::Forward));
        }
        let mut results: Vec<(Box<[u8]>, Box<[u8]>)> = Vec::new();
        for item in iter {
            let (key, value) = item?;
            let pre = &key[..prefix_bytes.len()];
            if pre != prefix_bytes {
                break;
            }
            results.push((key, value));
            if results.len() >= count {
                break;
            }
        }
        Ok(results)
    }
}

/// Implementation of the entity data source utilizing mokuroku to manage
/// secondary indices in an instance of RocksDB.
pub struct EntityDataSourceImpl {
    database: Database,
}

impl EntityDataSourceImpl {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, Error> {
        // the views provided by our Document implementations
        let views = vec![
            "by_checksum",
            "by_date",
            "by_location",
            "by_media_type",
            "by_tags",
            "by_year",
            "raw_locations",
            "newborn",
        ];
        std::fs::create_dir_all(&db_path)?;
        let database = Database::new(db_path, views, Box::new(mapper))?;
        Ok(Self { database })
    }

    fn fetch_all_locations(&self, view: &str) -> Result<Vec<Location>, Error> {
        let map = self.database.count_all_keys(view)?;
        let results: Vec<Location> = map
            .iter()
            .map(|t| {
                let raw = String::from_utf8((*t.0).to_vec()).unwrap();
                Location::str_deserialize(&raw)
            })
            .collect();
        Ok(results)
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
    fn get_asset_by_id(&self, asset_id: &str) -> Result<Asset, Error> {
        let db_key = format!("asset/{}", asset_id);
        let maybe_asset = self.database.get_asset(&db_key)?;
        maybe_asset.ok_or_else(|| anyhow!(format!("missing asset {}", asset_id)))
    }

    fn get_asset_by_digest(&self, digest: &str) -> Result<Option<Asset>, Error> {
        // secondary index keys are lowercase
        let digest = digest.to_lowercase();
        if let Some(qr_row) = self.database.query_one_by_key("by_checksum", digest)? {
            let asset_id_raw: Vec<u8> = Vec::from(qr_row.doc_id.as_ref());
            let asset_id_str = String::from_utf8(asset_id_raw).unwrap();
            return self.database.get_asset(&asset_id_str);
        }
        Ok(None)
    }

    fn put_asset(&self, asset: &Asset) -> Result<(), Error> {
        let key = format!("asset/{}", asset.key);
        self.database.put_asset(&key, asset)?;
        Ok(())
    }

    fn delete_asset(&self, asset_id: &str) -> Result<(), Error> {
        let key = format!("asset/{}", asset_id);
        self.database.delete_document(key.as_bytes())?;
        Ok(())
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
        let query_results = self.database.query_all_keys("by_location", locations)?;
        let search_results = convert_results(query_results);
        Ok(search_results)
    }

    fn query_by_media_type(&self, media_type: &str) -> Result<Vec<SearchResult>, Error> {
        // secondary index keys are lowercase
        let media_type = media_type.to_lowercase();
        let query_results = self.database.query_by_key("by_media_type", media_type)?;
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
        let epoch = Utc.timestamp_opt(0, 0).unwrap();
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
        // unfortunate naming due to existing index names
        self.count_all_keys("by_location")
    }

    fn raw_locations(&self) -> Result<Vec<Location>, Error> {
        self.fetch_all_locations("raw_locations")
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

    fn fetch_assets(&self, cursor: Option<String>, count: usize) -> Result<FetchedAssets, Error> {
        let prefixed_seek = cursor.map(|s| format!("asset/{}", s));
        let prefix_bytes = prefixed_seek.as_ref().map(|p| p.as_bytes().to_owned());
        // request one additional result and filter the one that matches the
        // cursor key, but also stopping when there are enough results
        let pairs = self.database.scan("asset/", prefixed_seek, count + 1)?;
        let mut results: Vec<Asset> = Vec::with_capacity(pairs.len());
        for (key, value) in pairs.into_iter() {
            if prefix_bytes
                .as_ref()
                .filter(|p| key.as_ref() == *p)
                .is_none()
            {
                results.push(Asset::from_bytes(key.as_ref(), value.as_ref())?);
                if results.len() >= count {
                    break;
                }
            }
        }
        // results are in lexicographical order so the last key will be used to
        // start scanning the next batch
        let cursor = results.last().map(|a| a.key.to_owned());
        Ok(FetchedAssets {
            assets: results,
            cursor,
        })
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
    fn from_bytes(key: &[u8], value: &[u8]) -> Result<Self, mokuroku::Error> {
        let mut de = serde_cbor::Deserializer::from_slice(value);
        let mut result = AssetModel::deserialize(&mut de)?;
        // remove the "asset/" key prefix added by the data source
        result.key = str::from_utf8(&key[6..])?.to_owned();
        Ok(result)
    }

    fn to_bytes(&self) -> Result<Vec<u8>, mokuroku::Error> {
        let mut encoded: Vec<u8> = Vec::new();
        let mut ser = serde_cbor::Serializer::new(&mut encoded);
        AssetModel::serialize(self, &mut ser)?;
        Ok(encoded)
    }

    fn map(&self, view: &str, emitter: &Emitter) -> Result<(), mokuroku::Error> {
        let value = SearchResult::new(self);
        let idv: Vec<u8> = value.to_bytes()?;
        if view == "by_tags" {
            for tag in &self.tags {
                let lower = tag.to_lowercase();
                emitter.emit(lower.as_bytes(), Some(&idv))?;
            }
        } else if view == "by_checksum" {
            let lower = self.checksum.to_lowercase();
            emitter.emit(lower.as_bytes(), None)?;
        } else if view == "by_media_type" {
            let lower = self.media_type.to_lowercase();
            emitter.emit(lower.as_bytes(), Some(&idv))?;
        } else if view == "by_location" {
            if let Some(loc) = self.location.as_ref() {
                for indexable in loc.indexable_values() {
                    emitter.emit(indexable.as_bytes(), Some(&idv))?;
                }
            }
        } else if view == "raw_locations" {
            // full location values separated by tabs and emitted as a single
            // index key with no value, intended for providing input completion
            //
            // this is an abuse of the indexing library in order to maintain the
            // index in the event of documents being updated or removed
            if let Some(loc) = self.location.as_ref() {
                if loc.has_values() {
                    let encoded = loc.str_serialize();
                    emitter.emit(encoded.as_bytes(), None)?;
                }
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
        } else if view == "newborn"
            && self.tags.is_empty()
            && self.caption.is_none()
            && self
                .location
                .as_ref()
                .and_then(|l| l.label.as_ref())
                .is_none()
        {
            // newborn assets have no caption, tags, or location label (city and
            // region are okay as those are filled in during import)
            //
            // use the import date for newborn, not the "best" date
            let import_date = encode_datetime(&self.import_date);
            emitter.emit(&import_date, Some(&idv))?;
        }
        Ok(())
    }
}

pub fn mapper(
    key: &[u8],
    value: &[u8],
    view: &str,
    emitter: &Emitter,
) -> Result<(), mokuroku::Error> {
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
    /// Location of the asset.
    #[serde(rename = "l")]
    pub location: Option<Location>,
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
    type Error = anyhow::Error;

    fn try_from(value: QueryResult) -> Result<Self, Self::Error> {
        let mut result = SearchResult::from_bytes(value.value.as_ref())?;
        // remove the "asset/" key prefix added by the data source
        let key_vec = (*value.doc_id).to_vec();
        result.asset_id = String::from_utf8(key_vec)?.split_off(6);
        Ok(result)
    }
}
