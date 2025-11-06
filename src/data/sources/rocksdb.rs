//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::data::sources::EntityDataSource;
use crate::domain::entities::{Asset, Dimensions, LabeledCount, Location, SearchResult};
use crate::domain::repositories::FetchedAssets;
use anyhow::{anyhow, Error};
use chrono::prelude::*;
use hashed_array_tree::HashedArrayTree;
use mokuroku::{base32, Document, Emitter, QueryResult};
use rocksdb::{Direction, IteratorMode, Options};
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

// The raw bytes of the key and value from the underlying database.
type RawKeyValue = (Box<[u8]>, Box<[u8]>);

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
    ) -> Result<HashedArrayTree<mokuroku::QueryResult>, Error> {
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
    ) -> Result<HashedArrayTree<mokuroku::QueryResult>, Error>
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
        Ok(db.query_all_keys_hat(view, exact_keys)?)
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
    ) -> Result<HashedArrayTree<mokuroku::QueryResult>, Error> {
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
    ) -> Result<HashedArrayTree<mokuroku::QueryResult>, Error> {
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
    ) -> Result<HashedArrayTree<mokuroku::QueryResult>, Error> {
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
    pub fn find_prefix(&self, prefix: &str) -> Result<HashedArrayTree<String>, Error> {
        let pre_bytes = prefix.as_bytes();
        // this only gets us started, we then have to check for the end of the range
        let db = self.db.lock().unwrap();
        let iter = db.db().prefix_iterator(pre_bytes);
        let mut results = HashedArrayTree::<String>::new();
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
    ) -> Result<Vec<RawKeyValue>, Error> {
        let prefix_bytes = prefix.as_bytes();
        let db = self.db.lock().unwrap();
        let mut iter = db.db().prefix_iterator(prefix_bytes);
        if let Some(from_str) = cursor {
            let from_bytes = from_str.as_bytes();
            iter.set_mode(IteratorMode::From(from_bytes, Direction::Forward));
        }
        let mut results: Vec<RawKeyValue> = Vec::new();
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

    fn query_by_tags(&self, tags: Vec<String>) -> Result<HashedArrayTree<SearchResult>, Error> {
        // secondary index keys are lowercase
        let tags: Vec<String> = tags.into_iter().map(|v| v.to_lowercase()).collect();
        let query_results = self.database.query_all_keys("by_tags", tags)?;
        let search_results = convert_results(query_results);
        Ok(search_results)
    }

    fn query_by_locations(
        &self,
        locations: Vec<String>,
    ) -> Result<HashedArrayTree<SearchResult>, Error> {
        // secondary index keys are lowercase
        let locations: Vec<String> = locations.into_iter().map(|v| v.to_lowercase()).collect();
        let query_results = self.database.query_all_keys("by_location", locations)?;
        let search_results = convert_results(query_results);
        Ok(search_results)
    }

    fn query_by_media_type(
        &self,
        media_type: &str,
    ) -> Result<HashedArrayTree<SearchResult>, Error> {
        // secondary index keys are lowercase
        let media_type = media_type.to_lowercase();
        let query_results = self.database.query_by_key("by_media_type", media_type)?;
        let search_results = convert_results(query_results);
        Ok(search_results)
    }

    fn query_before_date(
        &self,
        before: DateTime<Utc>,
    ) -> Result<HashedArrayTree<SearchResult>, Error> {
        let epoch = DateTime::UNIX_EPOCH;
        let before_key = encode_datetime(&before);
        let query_results = if epoch > before {
            // starting with a date that occurs before the epoch
            let min_utc = DateTime::<Utc>::MIN_UTC;
            let min_key = encode_datetime(&min_utc);
            self.database.query_range("by_date", min_key, before_key)?
        } else {
            // starting with a date that occurs after the epoch
            let mut results1 = self.database.query_less_than("by_date", before_key)?;
            let epoch_minus_key = b32hex_encode_i64(-1_i64);
            let min_utc = DateTime::<Utc>::MIN_UTC;
            let min_key = encode_datetime(&min_utc);
            let mut results2 = self
                .database
                .query_range("by_date", min_key, epoch_minus_key)?;
            results1.append(&mut results2);
            results1
        };
        let search_results = convert_results(query_results);
        Ok(search_results)
    }

    fn query_after_date(
        &self,
        after: DateTime<Utc>,
    ) -> Result<HashedArrayTree<SearchResult>, Error> {
        let epoch = DateTime::UNIX_EPOCH;
        let after_key = encode_datetime(&after);
        let query_results = if epoch <= after {
            // after the epoch, query range from then until end of time
            let max_utc = DateTime::<Utc>::MAX_UTC;
            let max_key = encode_datetime(&max_utc);
            self.database.query_range("by_date", after_key, max_key)?
        } else {
            // before the epoch, combine range from then until epoch and the
            // range from the epoch until the end of time
            let max_utc = DateTime::<Utc>::MAX_UTC;
            let max_key = encode_datetime(&max_utc);
            let epoch_minus_key = b32hex_encode_i64(-1_i64);
            let mut results1 = self
                .database
                .query_range("by_date", after_key, epoch_minus_key)?;
            let epoch_key = encode_datetime(&epoch);
            let mut results2 = self.database.query_range("by_date", epoch_key, max_key)?;
            results1.append(&mut results2);
            results1
        };
        let search_results = convert_results(query_results);
        Ok(search_results)
    }

    fn query_date_range(
        &self,
        after: DateTime<Utc>,
        before: DateTime<Utc>,
    ) -> Result<HashedArrayTree<SearchResult>, Error> {
        let epoch = DateTime::UNIX_EPOCH;
        let after_key = encode_datetime(&after);
        let before_key = encode_datetime(&before);
        let query_results = if after < epoch && before > epoch {
            // if the range crosses the epoch, combine two range queries
            let epoch_minus_key = b32hex_encode_i64(-1_i64);
            let mut results1 = self
                .database
                .query_range("by_date", after_key, epoch_minus_key)?;
            let epoch_key = encode_datetime(&epoch);
            let mut results2 = self
                .database
                .query_range("by_date", epoch_key, before_key)?;
            results1.append(&mut results2);
            results1
        } else {
            // otherwise the simple case
            self.database
                .query_range("by_date", after_key, before_key)?
        };
        let search_results = convert_results(query_results);
        Ok(search_results)
    }

    fn query_newborn(&self, after: DateTime<Utc>) -> Result<HashedArrayTree<SearchResult>, Error> {
        let epoch = DateTime::UNIX_EPOCH;
        let key = encode_datetime(&after);
        let query_results = if epoch > after {
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

    fn all_assets(&self) -> Result<HashedArrayTree<String>, Error> {
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

    fn store_assets(&self, incoming: Vec<Asset>) -> Result<(), Error> {
        // RocksDB is already super fast at writing key/value pairs, and other
        // than serializing the asset into bytes, there is nothing special to be
        // done here in terms of performance.
        for asset in incoming.iter() {
            self.put_asset(asset)?;
        }
        Ok(())
    }
}

/// Convert the database query results to our search results.
///
/// Silently drops any results that fail to deserialize.
fn convert_results(query_results: HashedArrayTree<QueryResult>) -> HashedArrayTree<SearchResult> {
    let search_results: HashedArrayTree<SearchResult> = query_results
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
    //
    // Base32hex encoded date/time values before the epoch are much larger than
    // even the maximum UTC time, and their value decreases as it approaches the
    // minimum UTC time (and likewise increase as they approach the epoch).
    //
    // min_utc : [56, 56, 56, 56, 47, 51, 52, 4a, 31, 4e, 4e, 30, 30, 3d, 3d, 3d]
    // epoch-1s: [56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 55, 3d, 3d, 3d]
    // epoch   : [30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 3d, 3d, 3d]
    // max_utc : [30, 30, 30, 30, 45, 54, 53, 51, 31, 39, 4c, 4e, 55, 3d, 3d, 3d]
    //
    // i64::MIN: -9223372036854775808
    // min_utc :    -8334601228800000 (milliseconds)
    // max_utc :     8210266876799999 (milliseconds)
    // i64::MAX:  9223372036854775807
    //
    let seconds = date.timestamp();
    let bytes = seconds.to_be_bytes().to_vec();
    base32::encode(&bytes)
}

/// Encode the i64 as a BigEndian base32hex encoded value.
fn b32hex_encode_i64(value: i64) -> Vec<u8> {
    let bytes = value.to_be_bytes().to_vec();
    base32::encode(&bytes)
}

impl Document for Asset {
    fn from_bytes(key: &[u8], value: &[u8]) -> Result<Self, mokuroku::Error> {
        use ciborium::Value;
        // remove the "asset/" key prefix added by the data source
        let key = str::from_utf8(&key[6..])?.to_owned();

        // process the raw Value into something easier to iterate over
        let raw_value: Value =
            ciborium::de::from_reader(value).map_err(|err| anyhow!("cbor read error: {}", err))?;
        let as_map: Vec<(Value, Value)> = raw_value
            .into_map()
            .map_err(|_| anyhow!("value: cbor into_map() error"))?;
        let mut values: HashMap<String, Value> = HashMap::new();
        for (name, value) in as_map.into_iter() {
            values.insert(
                name.into_text()
                    .map_err(|_| anyhow!("name: cbor into_text() error"))?,
                value,
            );
        }

        let checksum: String = values
            .remove("ch")
            .ok_or_else(|| anyhow!("missing 'ch' field"))?
            .into_text()
            .map_err(|_| anyhow!("ch: cbor into_text() error"))?;

        let filename: String = values
            .remove("fn")
            .ok_or_else(|| anyhow!("missing 'fn' field"))?
            .into_text()
            .map_err(|_| anyhow!("fn: cbor into_text() error"))?;

        let iv: ciborium::value::Integer = values
            .remove("sz")
            .ok_or_else(|| anyhow!("missing 'sz' field"))?
            .into_integer()
            .map_err(|_| anyhow!("sz: cbor into_integer() error"))?;
        let ii: i128 = ciborium::value::Integer::into(iv);
        let byte_length: u64 = ii as u64;

        let media_type: String = values
            .remove("mt")
            .ok_or_else(|| anyhow!("missing 'mt' field"))?
            .into_text()
            .map_err(|_| anyhow!("mt: cbor into_text() error"))?;

        let mut tags: Vec<String> = vec![];
        for v in values
            .remove("ta")
            .ok_or_else(|| anyhow!("missing 'ta' field"))?
            .into_array()
            .map_err(|_| anyhow!("ta: cbor into_array() error"))?
            .into_iter()
        {
            tags.push(
                v.into_text()
                    .map_err(|_| anyhow!("tag: cbor into_text() error"))?,
            );
        }

        let iv: ciborium::value::Integer = values
            .remove("id")
            .ok_or_else(|| anyhow!("missing 'id' field"))?
            .into_integer()
            .map_err(|_| anyhow!("id: cbor into_integer() error"))?;
        let ii: i128 = ciborium::value::Integer::into(iv);
        let import_date: DateTime<Utc> = DateTime::from_timestamp(ii as i64, 0).unwrap();

        let cp_value: Value = values
            .remove("cp")
            .ok_or_else(|| anyhow!("missing 'cp' field"))?;
        let caption: Option<String> = if cp_value.is_null() {
            None
        } else {
            Some(
                cp_value
                    .into_text()
                    .map_err(|_| anyhow!("cp: cbor into_text() error"))?,
            )
        };

        let lo_value: Value = values
            .remove("lo")
            .ok_or_else(|| anyhow!("missing 'lo' field"))?;
        let location: Option<Location> = if lo_value.is_null() {
            None
        } else if lo_value.is_text() {
            // location may be just a string if only a label is given
            let label = lo_value
                .into_text()
                .map_err(|_| anyhow!("lo: cbor into_text() error"))?;
            Some(Location {
                label: Some(label),
                city: None,
                region: None,
            })
        } else {
            // otherwise the location has three fields
            let as_map: Vec<(Value, Value)> = lo_value
                .into_map()
                .map_err(|_| anyhow!("lo: cbor into_map() error"))?;
            let mut label: Option<String> = None;
            let mut city: Option<String> = None;
            let mut region: Option<String> = None;
            for (name, value) in as_map.into_iter() {
                if name.as_text() == Some("l") {
                    if !value.is_null() {
                        label = Some(
                            value
                                .into_text()
                                .map_err(|_| anyhow!("l: cbor into_text() error"))?,
                        )
                    };
                } else if name.as_text() == Some("c") {
                    if !value.is_null() {
                        city = Some(
                            value
                                .into_text()
                                .map_err(|_| anyhow!("c: cbor into_text() error"))?,
                        )
                    };
                } else if name.as_text() == Some("r") && !value.is_null() {
                    region = Some(
                        value
                            .into_text()
                            .map_err(|_| anyhow!("r: cbor into_text() error"))?,
                    )
                };
            }
            Some(Location {
                label,
                city,
                region,
            })
        };

        let ud_value: Value = values
            .remove("ud")
            .ok_or_else(|| anyhow!("missing 'ud' field"))?;
        let user_date: Option<DateTime<Utc>> = if ud_value.is_null() {
            None
        } else {
            let iv = ud_value
                .into_integer()
                .map_err(|_| anyhow!("ud: cbor into_integer() error"))?;
            let ii: i128 = ciborium::value::Integer::into(iv);
            Some(DateTime::from_timestamp(ii as i64, 0).unwrap())
        };

        let od_value: Value = values
            .remove("od")
            .ok_or_else(|| anyhow!("missing 'od' field"))?;
        let original_date: Option<DateTime<Utc>> = if od_value.is_null() {
            None
        } else {
            let iv = od_value
                .into_integer()
                .map_err(|_| anyhow!("od: cbor into_integer() error"))?;
            let ii: i128 = ciborium::value::Integer::into(iv);
            Some(DateTime::from_timestamp(ii as i64, 0).unwrap())
        };

        let dm_value: Value = values
            .remove("dm")
            .ok_or_else(|| anyhow!("missing 'dm' field"))?;
        let dimensions: Option<Dimensions> = if dm_value.is_null() {
            None
        } else {
            let mut as_arr: Vec<Value> = dm_value
                .into_array()
                .map_err(|_| anyhow!("dm: cbor into_array() error"))?;
            let iv = as_arr
                .remove(0)
                .into_integer()
                .map_err(|_| anyhow!("dm.0: cbor into_integer() error"))?;
            let w: i128 = ciborium::value::Integer::into(iv);
            let iv = as_arr
                .remove(0)
                .into_integer()
                .map_err(|_| anyhow!("dm.1: cbor into_integer() error"))?;
            let h: i128 = ciborium::value::Integer::into(iv);
            Some(Dimensions(w as u32, h as u32))
        };

        Ok(Asset {
            key,
            checksum,
            filename,
            byte_length,
            media_type,
            tags,
            import_date,
            caption,
            location,
            user_date,
            original_date,
            dimensions,
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>, mokuroku::Error> {
        use ciborium::Value;
        let mut encoded: Vec<u8> = Vec::new();
        // Emit everything except the key since that is part of the data store
        // already, and use short names for an overall smaller size.
        let mut fields: Vec<(Value, Value)> = vec![
            // checksum
            (Value::Text("ch".into()), Value::Text(self.checksum.clone())),
            // filename
            (Value::Text("fn".into()), Value::Text(self.filename.clone())),
            // byte_length
            (
                Value::Text("sz".into()),
                Value::Integer(self.byte_length.into()),
            ),
            // media_type
            (
                Value::Text("mt".into()),
                Value::Text(self.media_type.clone()),
            ),
            // tags
            (
                Value::Text("ta".into()),
                Value::Array(
                    self.tags
                        .iter()
                        .map(|t| Value::Text(t.to_owned()))
                        .collect(),
                ),
            ),
            // import_date
            (
                Value::Text("id".into()),
                Value::Integer(self.import_date.timestamp().into()),
            ),
        ];

        // caption
        if let Some(ref cp) = self.caption {
            fields.push((Value::Text("cp".into()), Value::Text(cp.to_owned())));
        } else {
            fields.push((Value::Text("cp".into()), Value::Null));
        }

        // location
        if let Some(ref loc) = self.location {
            // if location has only a label, emit as a string
            if loc.label.is_some() && loc.city.is_none() && loc.region.is_none() {
                fields.push((
                    Value::Text("lo".into()),
                    Value::Text(loc.label.as_ref().unwrap().to_owned()),
                ));
            } else {
                let mut parts: Vec<(Value, Value)> = vec![];
                if let Some(ref label) = loc.label {
                    parts.push((Value::Text("l".into()), Value::Text(label.to_owned())));
                } else {
                    parts.push((Value::Text("l".into()), Value::Null));
                }
                if let Some(ref city) = loc.city {
                    parts.push((Value::Text("c".into()), Value::Text(city.to_owned())));
                } else {
                    parts.push((Value::Text("c".into()), Value::Null));
                }
                if let Some(ref region) = loc.region {
                    parts.push((Value::Text("r".into()), Value::Text(region.to_owned())));
                } else {
                    parts.push((Value::Text("r".into()), Value::Null));
                }
                fields.push((Value::Text("lo".into()), Value::Map(parts)));
            }
        } else {
            fields.push((Value::Text("lo".into()), Value::Null));
        }

        // user_date
        if let Some(ud) = self.user_date {
            fields.push((
                Value::Text("ud".into()),
                Value::Integer(ud.timestamp().into()),
            ));
        } else {
            fields.push((Value::Text("ud".into()), Value::Null));
        }

        // original_date
        if let Some(od) = self.original_date {
            fields.push((
                Value::Text("od".into()),
                Value::Integer(od.timestamp().into()),
            ));
        } else {
            fields.push((Value::Text("od".into()), Value::Null));
        }

        // dimensions
        if let Some(ref dim) = self.dimensions {
            fields.push((
                Value::Text("dm".into()),
                Value::Array(vec![
                    Value::Integer(dim.0.into()),
                    Value::Integer(dim.1.into()),
                ]),
            ));
        } else {
            fields.push((Value::Text("dm".into()), Value::Null));
        }
        let doc = Value::Map(fields);
        ciborium::ser::into_writer(&doc, &mut encoded)
            .map_err(|err| anyhow!("cbor write error: {}", err))?;
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
            emitter.emit(bytes, None)?;
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

impl SearchResult {
    fn from_bytes(value: &[u8]) -> Result<Self, Error> {
        use ciborium::Value;
        // process the raw Value into something easier to iterate over
        let raw_value: Value =
            ciborium::de::from_reader(value).map_err(|err| anyhow!("cbor read error: {}", err))?;
        let as_map: Vec<(Value, Value)> = raw_value
            .into_map()
            .map_err(|_| anyhow!("value: cbor into_map() error"))?;
        let mut values: HashMap<String, Value> = HashMap::new();
        for (name, value) in as_map.into_iter() {
            values.insert(
                name.into_text()
                    .map_err(|_| anyhow!("name: cbor into_text() error"))?,
                value,
            );
        }

        let filename: String = values
            .remove("n")
            .ok_or_else(|| anyhow!("missing 'n' field"))?
            .into_text()
            .map_err(|_| anyhow!("n: cbor into_text() error"))?;

        let media_type: String = values
            .remove("m")
            .ok_or_else(|| anyhow!("missing 'm' field"))?
            .into_text()
            .map_err(|_| anyhow!("m: cbor into_text() error"))?;

        let lo_value: Value = values
            .remove("l")
            .ok_or_else(|| anyhow!("missing 'l' field"))?;
        let location: Option<Location> = if lo_value.is_null() {
            None
        } else {
            let as_map: Vec<(Value, Value)> = lo_value
                .into_map()
                .map_err(|_| anyhow!("l: cbor into_map() error"))?;
            let mut label: Option<String> = None;
            let mut city: Option<String> = None;
            let mut region: Option<String> = None;
            for (name, value) in as_map.into_iter() {
                if name.as_text() == Some("l") {
                    if !value.is_null() {
                        label = Some(
                            value
                                .into_text()
                                .map_err(|_| anyhow!("l: cbor into_text() error"))?,
                        )
                    };
                } else if name.as_text() == Some("c") {
                    if !value.is_null() {
                        city = Some(
                            value
                                .into_text()
                                .map_err(|_| anyhow!("c: cbor into_text() error"))?,
                        )
                    };
                } else if name.as_text() == Some("r") && !value.is_null() {
                    region = Some(
                        value
                            .into_text()
                            .map_err(|_| anyhow!("r: cbor into_text() error"))?,
                    )
                };
            }
            Some(Location {
                label,
                city,
                region,
            })
        };

        let iv: ciborium::value::Integer = values
            .remove("d")
            .ok_or_else(|| anyhow!("missing 'd' field"))?
            .into_integer()
            .map_err(|_| anyhow!("d: cbor into_integer() error"))?;
        let ii: i128 = ciborium::value::Integer::into(iv);
        let datetime: DateTime<Utc> = DateTime::from_timestamp(ii as i64, 0).unwrap();

        Ok(SearchResult {
            asset_id: Default::default(),
            filename,
            media_type,
            location,
            datetime,
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        use ciborium::Value;
        let mut encoded: Vec<u8> = Vec::new();
        // Emit everything except the asset_id since that is part of the data
        // store already, and use short names for an overall smaller size.
        let mut fields: Vec<(Value, Value)> = vec![];

        // filename
        fields.push((Value::Text("n".into()), Value::Text(self.filename.clone())));

        // media_type
        fields.push((
            Value::Text("m".into()),
            Value::Text(self.media_type.clone()),
        ));

        // location
        if let Some(ref loc) = self.location {
            let mut parts: Vec<(Value, Value)> = vec![];
            if let Some(ref label) = loc.label {
                parts.push((Value::Text("l".into()), Value::Text(label.to_owned())));
            } else {
                parts.push((Value::Text("l".into()), Value::Null));
            }
            if let Some(ref city) = loc.city {
                parts.push((Value::Text("c".into()), Value::Text(city.to_owned())));
            } else {
                parts.push((Value::Text("c".into()), Value::Null));
            }
            if let Some(ref region) = loc.region {
                parts.push((Value::Text("r".into()), Value::Text(region.to_owned())));
            } else {
                parts.push((Value::Text("r".into()), Value::Null));
            }
            fields.push((Value::Text("l".into()), Value::Map(parts)));
        } else {
            fields.push((Value::Text("l".into()), Value::Null));
        }

        // datetime
        fields.push((
            Value::Text("d".into()),
            Value::Integer(self.datetime.timestamp().into()),
        ));

        let doc = Value::Map(fields);
        ciborium::ser::into_writer(&doc, &mut encoded)
            .map_err(|err| anyhow!("cbor write error: {}", err))?;
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

#[cfg(test)]
mod test {
    use super::*;

    // Asset does not implement a full Eq, it only compares the key
    fn compare_assets(a: &Asset, b: &Asset) {
        assert_eq!(a.key, b.key);
        assert_eq!(a.checksum, b.checksum);
        assert_eq!(a.filename, b.filename);
        assert_eq!(a.byte_length, b.byte_length);
        assert_eq!(a.media_type, b.media_type);
        assert_eq!(a.tags, b.tags);
        assert_eq!(a.import_date, b.import_date);
        assert_eq!(a.caption, b.caption);
        assert_eq!(a.location, b.location);
        assert_eq!(a.user_date, b.user_date);
        assert_eq!(a.original_date, b.original_date);
        assert_eq!(a.dimensions, b.dimensions);
    }

    // SearchResult does not serialize the asset identifier
    fn compare_results(a: &SearchResult, b: &SearchResult) {
        assert_eq!(a.filename, b.filename);
        assert_eq!(a.media_type, b.media_type);
        assert_eq!(a.location, b.location);
        assert_eq!(a.datetime, b.datetime);
    }

    // generate a sha1-XXX style hash of the given input
    fn compute_key_hash(key: &str) -> String {
        use sha1::{Digest, Sha1};
        let mut hasher = Sha1::new();
        hasher.update(key.as_bytes());
        let digest = hasher.finalize();
        format!("sha1-{:x}", digest)
    }

    /// Construct an asset in which every field is populated.
    pub fn build_complete_asset(key: &str) -> Asset {
        let mut asset = build_minimal_asset(key);
        asset.tags = vec!["beach".into(), "birds".into(), "waves".into()];
        asset.caption = Some("fun in the sun".into());
        asset.location = Some(Location::with_parts("beach", "Honolulu", "Hawaii"));
        asset.user_date = Some(
            Utc.with_ymd_and_hms(2018, 6, 1, 18, 15, 0)
                .single()
                .unwrap(),
        );
        asset.original_date = Some(
            Utc.with_ymd_and_hms(2018, 5, 30, 12, 30, 0)
                .single()
                .unwrap(),
        );
        asset.dimensions = Some(Dimensions(1024, 768));
        asset
    }

    /// Construct an asset with only required fields populated.
    pub fn build_minimal_asset(key: &str) -> Asset {
        let checksum = compute_key_hash(key);
        let now_s = Utc::now().timestamp();
        let import_date = DateTime::from_timestamp(now_s, 0).unwrap();
        Asset {
            key: key.to_owned(),
            checksum,
            filename: "img_2468.jpg".to_owned(),
            byte_length: 1048576,
            media_type: "image/jpeg".to_owned(),
            tags: vec![],
            import_date,
            caption: None,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        }
    }

    #[test]
    fn test_asset_from_to_bytes() {
        // test with bare minimum asset
        let asset = build_minimal_asset("minimal");
        let bytes = asset.to_bytes().unwrap();
        let key = "asset/minimal";
        let actual = Asset::from_bytes(key.as_bytes(), &bytes).unwrap();
        compare_assets(&asset, &actual);

        // test with maximally complete asset
        let asset = build_complete_asset("maximal");
        let bytes = asset.to_bytes().unwrap();
        let key = "asset/maximal";
        let actual = Asset::from_bytes(key.as_bytes(), &bytes).unwrap();
        compare_assets(&asset, &actual);

        // test with asset that has only location label
        let mut asset = build_minimal_asset("labelonly");
        asset.location = Some(Location::new("hong kong"));
        let bytes = asset.to_bytes().unwrap();
        let key = "asset/labelonly";
        let actual = Asset::from_bytes(key.as_bytes(), &bytes).unwrap();
        compare_assets(&asset, &actual);

        // test with asset that has only location city
        let mut asset = build_minimal_asset("cityonly");
        asset.location = Some(Location {
            label: None,
            city: Some("Paris".into()),
            region: None,
        });
        let bytes = asset.to_bytes().unwrap();
        let key = "asset/cityonly";
        let actual = Asset::from_bytes(key.as_bytes(), &bytes).unwrap();
        compare_assets(&asset, &actual);

        // test with asset that has only location region
        let mut asset = build_minimal_asset("regiononly");
        asset.location = Some(Location {
            label: None,
            city: None,
            region: Some("Oregon".into()),
        });
        let bytes = asset.to_bytes().unwrap();
        let key = "asset/regiononly";
        let actual = Asset::from_bytes(key.as_bytes(), &bytes).unwrap();
        compare_assets(&asset, &actual);
    }

    #[test]
    fn test_searchresult_from_to_bytes() {
        let now_s = Utc::now().timestamp();
        let datetime = DateTime::from_timestamp(now_s, 0).unwrap();
        // test with result that has only location label
        let expected = SearchResult {
            asset_id: "".into(),
            filename: "img_1234.jpg".into(),
            media_type: "image/jpeg".into(),
            location: Some(Location::new("beach")),
            datetime,
        };
        let bytes = expected.to_bytes().unwrap();
        let actual = SearchResult::from_bytes(&bytes).unwrap();
        compare_results(&expected, &actual);

        // test with result that has only location city
        let expected = SearchResult {
            asset_id: "".into(),
            filename: "img_1234.jpg".into(),
            media_type: "image/jpeg".into(),
            location: Some(Location::with_parts("", "Paris", "")),
            datetime,
        };
        let bytes = expected.to_bytes().unwrap();
        let actual = SearchResult::from_bytes(&bytes).unwrap();
        compare_results(&expected, &actual);

        // test with result that has only location region
        let expected = SearchResult {
            asset_id: "".into(),
            filename: "img_1234.jpg".into(),
            media_type: "image/jpeg".into(),
            location: Some(Location::with_parts("", "", "Oregon")),
            datetime,
        };
        let bytes = expected.to_bytes().unwrap();
        let actual = SearchResult::from_bytes(&bytes).unwrap();
        compare_results(&expected, &actual);
    }
}
