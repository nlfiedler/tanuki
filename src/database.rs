//
// Copyright (c) 2020 Nathan Fiedler
//

//! The `database` module provides high-level functions for storing and
//! retrieving the various core data structures to a database. This module
//! performs the serde functions to convert the structures to a format suitable
//! for on-disk storage.

use super::core;
use failure::Error;
use lazy_static::lazy_static;
use rocksdb::backup::{BackupEngine, BackupEngineOptions};
use rocksdb::Options;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str;
use std::sync::{Arc, Mutex, Weak};

lazy_static! {
    // Keep a map of weakly held references to shared DB instances. RocksDB
    // itself is thread-safe for get/put/write, and the DB type implements Send
    // and Sync. We just need to make sure the instance is eventually closed
    // when the last reference is dropped.
    //
    // The key is the path to the database files.
    //
    // If the Mutex proves to be problematic, switch to ReentrantMutex in the
    // parking_lot crate, which allows recursive locking.
    static ref DBASE_REFS: Mutex<HashMap<PathBuf, Weak<Mutex<mokuroku::Database>>>> = Mutex::new(HashMap::new());
}

///
/// An instance of the database for reading and writing records to disk.
///
pub struct Database {
    /// RocksDB instance.
    db: Arc<Mutex<mokuroku::Database>>,
}

// Mark the Database as a valid context type for Juniper.
// impl juniper::Context for Database {}

impl Database {
    ///
    /// Create an instance of Database using the given path for storage. Will
    /// reuse an existing `DB` instance for the given path, if one has already
    /// been opened.
    ///
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, Error> {
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
        let views = vec![
            "by_checksum",
            "by_date",
            "by_filename",
            "by_location",
            "by_mimetype",
            "by_tag",
        ];
        let db = mokuroku::Database::open(db_path.as_ref(), views, Box::new(core::mapper), opts)?;
        let arc = Arc::new(Mutex::new(db));
        db_refs.insert(buf, Arc::downgrade(&arc));
        Ok(Self { db: arc })
    }

    // ///
    // /// Return the path to the database files.
    // ///
    // pub fn get_path(&self) -> &Path {
    //     let db = self.db.lock().unwrap();
    //     db.db().path()
    // }

    ///
    /// Create a backup of the database at the given path.
    ///
    pub fn create_backup<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let backup_opts = BackupEngineOptions::default();
        let mut backup_engine = BackupEngine::open(&backup_opts, path.as_ref())?;
        let db = self.db.lock().unwrap();
        backup_engine.create_new_backup(db.db())?;
        backup_engine.purge_old_backups(1)?;
        Ok(())
    }

    ///
    /// Restore the database from the backup path to the given db path.
    ///
    pub fn restore_from_backup<P: AsRef<Path>>(backup_path: P, db_path: P) -> Result<(), Error> {
        let backup_opts = BackupEngineOptions::default();
        let mut backup_engine = BackupEngine::open(&backup_opts, &backup_path).unwrap();
        let mut restore_option = rocksdb::backup::RestoreOptions::default();
        restore_option.set_keep_log_files(true);
        backup_engine.restore_from_latest_backup(&db_path, &db_path, &restore_option)?;
        Ok(())
    }

    ///
    /// Insert the value if the database does not already contain the given key.
    ///
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
    pub fn get_document(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        let db = self.db.lock().unwrap();
        let result = db.db().get(key)?;
        Ok(result)
    }

    ///
    /// Put the key/value pair into the database.
    ///
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
        db.delete(key)
    }

    ///
    /// Put the given asset into the database.
    ///
    pub fn put_asset(&self, asset: &core::Asset) -> Result<(), Error> {
        let key = format!("asset/{}", asset.key);
        let mut db = self.db.lock().unwrap();
        db.put(key, asset)
    }

    ///
    /// Retrieve the asset by the given key, returning None if not found.
    ///
    pub fn get_asset(&self, key: &str) -> Result<Option<core::Asset>, Error> {
        let db_key = format!("asset/{}", key);
        let db = self.db.lock().unwrap();
        db.get(db_key)
    }

    // ///
    // /// Retrieve all of the assets in the database.
    // ///
    // pub fn get_all_assets(&self) -> Result<Vec<core::Asset>, Error> {
    //     let assets = self.fetch_prefix("asset/")?;
    //     let mut results: Vec<core::Asset> = Vec::new();
    //     for (key, value) in assets {
    //         let mut serde_result: core::Asset = serde_cbor::from_slice(&value)?;
    //         // strip the "asset/" prefix from the key
    //         serde_result.key = key[6..].to_string();
    //         results.push(serde_result);
    //     }
    //     Ok(results)
    // }

    // ///
    // /// Delete the given asset from the database.
    // ///
    // pub fn delete_asset(&self, key: &str) -> Result<(), Error> {
    //     let key = format!("asset/{}", key);
    //     self.delete_document(key.as_bytes())
    // }

    ///
    /// Count those keys that start with the given prefix.
    ///
    pub fn count_prefix(&self, prefix: &str) -> Result<usize, Error> {
        let pre_bytes = prefix.as_bytes();
        // this only gets us started, we then have to check for the end of the range
        let db = self.db.lock().unwrap();
        let iter = db.db().prefix_iterator(pre_bytes);
        let mut count = 0;
        for (key, _value) in iter {
            let pre = &key[..pre_bytes.len()];
            if pre != pre_bytes {
                break;
            }
            count += 1;
        }
        Ok(count)
    }

    ///
    /// Find all those keys that start with the given prefix.
    ///
    pub fn find_prefix(&self, prefix: &str) -> Result<Vec<String>, Error> {
        let pre_bytes = prefix.as_bytes();
        // this only gets us started, we then have to check for the end of the range
        let db = self.db.lock().unwrap();
        let iter = db.db().prefix_iterator(pre_bytes);
        let mut results: Vec<String> = Vec::new();
        for (key, _value) in iter {
            let pre = &key[..pre_bytes.len()];
            if pre != pre_bytes {
                break;
            }
            let key_str = str::from_utf8(&key)?;
            results.push(key_str.to_owned());
        }
        Ok(results)
    }

    ///
    /// Fetch the key/value pairs for those keys that start with the given
    /// prefix.
    ///
    pub fn fetch_prefix(&self, prefix: &str) -> Result<HashMap<String, Box<[u8]>>, Error> {
        let pre_bytes = prefix.as_bytes();
        // this only gets us started, we then have to check for the end of the range
        let db = self.db.lock().unwrap();
        let iter = db.db().prefix_iterator(pre_bytes);
        let mut results: HashMap<String, Box<[u8]>> = HashMap::new();
        for (key, value) in iter {
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
