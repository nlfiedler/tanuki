//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::Asset;
use anyhow::Error;
use rocksdb::backup::{self, BackupEngine, BackupEngineOptions};
use rocksdb::{Direction, Env, IteratorMode, Options};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str;
use std::sync::{Arc, LazyLock, Mutex, Weak};

// Keep a map of weakly held references to shared DB instances. RocksDB itself
// is thread-safe for get/put/write, and the DB type implements Send and Sync.
// We just need to make sure the instance is eventually closed when the last
// reference is dropped.
//
// The key is the path to the database files.
//
// Need a mutex on the database to allow mutation (mokuroku requires this for
// managing the column families). If the Mutex proves to be problematic, switch
// to ReentrantMutex in the parking_lot crate, which allows recursive locking.
static DBASE_REFS: LazyLock<Mutex<HashMap<PathBuf, Weak<Mutex<mokuroku::Database>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

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
        if let Some(weak) = db_refs.get(db_path.as_ref()) {
            if let Some(arc) = weak.upgrade() {
                return Ok(Self { db: arc });
            }
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
        db_refs.insert(buf, Arc::downgrade(&arc));
        Ok(Self { db: arc })
    }

    ///
    /// Create a backup of the database at the given path.
    ///
    #[allow(dead_code)]
    pub fn create_backup<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let backup_opts = BackupEngineOptions::new(path.as_ref())?;
        let env = Env::new()?;
        let mut backup_engine = BackupEngine::open(&backup_opts, &env)?;
        let db = self.db.lock().unwrap();
        backup_engine.create_new_backup(db.db())?;
        backup_engine.purge_old_backups(1)?;
        Ok(())
    }

    ///
    /// Restore the database from the backup path to the given db path.
    ///
    #[allow(dead_code)]
    pub fn restore_from_backup<P: AsRef<Path>>(backup_path: P, db_path: P) -> Result<(), Error> {
        let backup_opts = BackupEngineOptions::new(&backup_path)?;
        let env = Env::new()?;
        let mut backup_engine = BackupEngine::open(&backup_opts, &env)?;
        let mut restore_option = backup::RestoreOptions::default();
        restore_option.set_keep_log_files(true);
        backup_engine.restore_from_latest_backup(&db_path, &db_path, &restore_option)?;
        Ok(())
    }

    ///
    /// Insert the value if the database does not already contain the given key.
    ///
    #[allow(dead_code)]
    pub fn insert_document(&self, key: &[u8], value: &[u8]) -> Result<(), Error> {
        let db = self.db.lock().unwrap();
        let existing = db.db().get(key)?;
        if existing.is_none() {
            db.db().put(key, value)?;
        }
        Ok(())
    }

    ///
    /// Retrieve the value with the given key.
    ///
    #[allow(dead_code)]
    pub fn get_document(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        let db = self.db.lock().unwrap();
        let result = db.db().get(key)?;
        Ok(result)
    }

    ///
    /// Put the key/value pair into the database.
    ///
    #[allow(dead_code)]
    pub fn put_document(&self, key: &[u8], value: &[u8]) -> Result<(), Error> {
        let db = self.db.lock().unwrap();
        db.db().put(key, value)?;
        Ok(())
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
    /// optionally seeking to the key given in `seek_from`, and returning up to
    /// `count` items.
    ///
    /// Unlike other queries, this function returns the raw key/value pairs.
    ///
    pub fn scan(
        &self,
        prefix: &str,
        seek_from: Option<String>,
        count: usize,
    ) -> Result<Vec<(Box<[u8]>, Box<[u8]>)>, Error> {
        let prefix_bytes = prefix.as_bytes();
        let db = self.db.lock().unwrap();
        let mut iter = db.db().prefix_iterator(prefix_bytes);
        if let Some(from_str) = seek_from {
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

    ///
    /// Fetch the key/value pairs for those keys that start with the given
    /// prefix.
    ///
    #[allow(dead_code)]
    pub fn fetch_prefix(&self, prefix: &str) -> Result<HashMap<String, Box<[u8]>>, Error> {
        let pre_bytes = prefix.as_bytes();
        // this only gets us started, we then have to check for the end of the range
        let db = self.db.lock().unwrap();
        let iter = db.db().prefix_iterator(pre_bytes);
        let mut results: HashMap<String, Box<[u8]>> = HashMap::new();
        for item in iter {
            let (key, value) = item?;
            let pre = &key[..pre_bytes.len()];
            if pre != pre_bytes {
                break;
            }
            let key_str = str::from_utf8(&key)?;
            results.insert(key_str.to_owned(), value);
        }
        Ok(results)
    }
}
